use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::{
        math as na,
        timing::{Time},
        transform::{TransformBundle, Transform},
        Named
    },
    ecs::{Entity, Component, System, Join, VecStorage, NullStorage},
    ecs::prelude::{
        Read,
        ReadStorage,
        WriteStorage,
        Resources,
        SystemData
    },
    input::{InputBundle, InputHandler, StringBindings},
    prelude::*,
    renderer::{
        plugins::{RenderFlat2D, RenderToWindow},
        types::DefaultBackend,
        camera::Projection,
        Camera,
        ImageFormat,
        SpriteSheetFormat,
        RenderingBundle,
        SpriteRender, SpriteSheet,
        Texture, Transparent
    },
    utils::application_root_dir,
    window::ScreenDimensions
};

mod animation;
mod character;

use crate::character::{Player, PlayerSystem};
use crate::animation::{SpriteAnimation, AnimationSystem, AnimationType, AnimationData, AnimationResource};

type Vector3 = na::Vector3<f32>;

const SCALE_FACTOR: f32 = 3.;

fn load_sprite_sheet(world: &World, file_name: &str) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(format!("{}.png", file_name), ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}.ron", file_name),
        SpriteSheetFormat(texture_handle),
        (),
        &sprite_sheet_store
    )
}

fn init_player_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(30., 30., 0.);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0
    };
    let physics = Physics {
        acceleration: Vector3::new(0.,0.,0.),
        velocity: Vector3::new(0.,0.,0.),
        mass: 1.,
        friction: 0.5,
        is_jumping: false
    };
    let player = Player::new();
    let animation_data =  world.read_resource::<AnimationResource>().player_idle.clone();
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(physics)
        .with(player)
        .with(AnimationType::Idle)
        .with(Transparent)
        .build();
}

fn init_enemy_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(200., 30., -1.);
    sprite_transform.set_rotation_y_axis(std::f32::consts::PI);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0
    };
    let animation_data =  world.read_resource::<AnimationResource>().player_attack_0.clone();
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(Physics::default())
        .with(AnimationType::Attack(0))
        .with(Transparent)
        .build();
}

fn init_camera(world: &mut World) {
    let (width, height) = {
        let dim = world.read_resource::<ScreenDimensions>();
        (dim.width()/SCALE_FACTOR, dim.height()/SCALE_FACTOR)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(width/2., height/2., 1.);
    let camera = Camera::standard_2d(width, height);
    world.create_entity()
        .with(camera)
        .with(transform)
        .build();
}

#[derive(Default, Debug)]
struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, mut data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;
        world.add_resource(
            AnimationResource {
                player_idle: AnimationData::new(vec![0,1,2,3], 1./8., AnimationType::Idle, true),
                player_run: AnimationData::new(vec![4,5,6,7,8,9], 1./10., AnimationType::Run, true),
                player_jump: AnimationData::new(vec![10,11,12,13], 1./12., AnimationType::Jump, false),
                player_attack_0: AnimationData::new(vec![14,15,16,17,18], 1./10., AnimationType::Attack(0), true),
                player_attack_1: AnimationData::new(vec![19,20,21,22,23,24], 1./12., AnimationType::Attack(1), true),
                player_attack_2: AnimationData::new(vec![25,26,27,28,29,30], 1./10., AnimationType::Attack(2), true)
            }
        );
        let player_sprite_sheet_handle = load_sprite_sheet(&world, "sprite_sheet");
        init_player_sprite(&mut world, &player_sprite_sheet_handle);
        init_enemy_sprite(&mut world, &player_sprite_sheet_handle);
        init_camera(&mut world);
    }
}

#[derive(Debug)]
pub struct Physics {
    acceleration: Vector3,
    velocity: Vector3,
    mass: f32,
    friction: f32,
    is_jumping: bool
}
impl Component for Physics {
    type Storage = VecStorage<Self>;
}
impl Default for Physics {
    fn default() -> Physics {
        Physics {
            acceleration: Vector3::new(0., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            mass: 0.,
            friction: 0.,
            is_jumping: false
        }
    }
}

struct PhysicsSystem;
impl <'a> System<'a> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'a, Physics>,
        WriteStorage<'a, Transform>,
        Read<'a, Time>
    );
    fn run(&mut self, (mut physics_set, mut transform, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (physics, transform) in (&mut physics_set, &mut transform).join() {
            //
            if transform.translation().y > 20. || physics.velocity.y > 0. {
                physics.acceleration.y = -9.8;
                physics.velocity.y += physics.acceleration.y*dt;
            } else if physics.is_jumping {
                //we hit the ground, reset jumping flag
                physics.is_jumping = false;
                physics.acceleration.y = 0.;
                physics.velocity.y = 0.;
            }
            physics.velocity.x += physics.acceleration.x*dt;
            physics.velocity.x -= physics.velocity.x*physics.friction*9.8*dt;
            //clamp translation so new coordinates are always on the screen
            let mut new_translation = transform.translation() + physics.velocity;
            new_translation.x = new_translation.x.min(800./SCALE_FACTOR).max(0.);
            new_translation.y = new_translation.y.min(600./SCALE_FACTOR).max(20.);
            transform.set_translation(new_translation);
        }
    }
}

struct MovementSystem;
impl <'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Physics>,
        Read<'a, InputHandler<StringBindings>>,
        Read<'a, Time>
    );
    fn run(&mut self, (mut players, mut physics_set, input, time): Self::SystemData) {
        let dt = time.delta_seconds();
        let (cx, cy, attack, jump) = (
            input.axis_value("x").unwrap(),
            input.axis_value("y").unwrap(),
            input.action_is_down("attack").unwrap(),
            input.action_is_down("jump").unwrap()
        );
        for (player, physics) in (&mut players, &mut physics_set).join() {
            physics.acceleration.x = cx as f32 * 10.;
            if attack && !player.is_attacking && !physics.is_jumping  {
                player.is_attacking = true;
            } else if jump && !physics.is_jumping {
                physics.is_jumping = true;
                physics.velocity.y = 5.;
            }
        }
    }
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("assets");
    let config_dir = app_root.join("config");
    let display_config_path = config_dir.join("display.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(
            InputBundle::<StringBindings>::new()
                .with_bindings_from_file(config_dir.join("input.ron"))?
        )?
        .with(MovementSystem, "movement_system", &[])
        .with(PhysicsSystem, "physics_system", &[])
        .with(PlayerSystem, "player_system", &[])
        .with(AnimationSystem, "animation_system", &[])
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderFlat2D::default()),
        )?;

    let mut game = Application::new(assets_dir, MyState, game_data)?;
    game.run();

    Ok(())
}
