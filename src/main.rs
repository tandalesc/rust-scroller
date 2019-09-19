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

use std::collections::HashMap;

mod animation;
use crate::animation::{SpriteAnimation, AnimationSystem, AnimationType, AnimationData, AnimationResource};

type Vector3 = na::Vector3<f32>;

const SCALE_FACTOR: f32 = 3.;

fn load_sprite_sheet(world: &World) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load("sprite_sheet.png", ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        "sprite_sheet.ron",
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
        friction: 0.4
    };
    let animation_data =  world.read_resource::<AnimationResource>().player_idle.clone();
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(physics)
        .with(Transparent)
        .with(Player)
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
    let physics = Physics::default();
    let animation_data =  world.read_resource::<AnimationResource>().player_idle.clone();
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(physics)
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
                player_idle: AnimationData::new(vec![0,0,0,0,0,1,2,3,4,5,6,7,8,9], 1./12., AnimationType::Idle),
                player_run: AnimationData::new(vec![10,11,12,13,14,15,16,17], 1./12., AnimationType::Run)
            }
        );
        let sprite_sheet_handle = load_sprite_sheet(&world);
        init_player_sprite(&mut world, &sprite_sheet_handle);
        init_enemy_sprite(&mut world, &sprite_sheet_handle);
        init_camera(&mut world);
    }
}

#[derive(Default, Debug)]
struct Player;
impl Component for Player {
    type Storage = NullStorage<Self>;
}

#[derive(Debug)]
pub struct Physics {
    acceleration: Vector3,
    velocity: Vector3,
    mass: f32,
    friction: f32
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
            friction: 0.
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
            if transform.translation().y > 20. {
                physics.acceleration.y = -9.8;
                physics.velocity.y += physics.acceleration.y*dt;
            } else if physics.velocity.y < 0. {
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
        ReadStorage<'a, Player>,
        WriteStorage<'a, Physics>,
        Read<'a, InputHandler<StringBindings>>
    );
    fn run(&mut self, (player, mut physics_set, input): Self::SystemData) {
        let (rx, ry) = (
            input.axis_value("r_x").unwrap(),
            input.axis_value("r_y").unwrap()
        );
        for (_, physics) in (&player, &mut physics_set).join() {
            physics.acceleration.x = rx as f32 * 10.;
            if ry == 1. && physics.velocity.y == 0. {
                physics.velocity.y = 5.0;
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
        .with(PhysicsSystem, "physics_system", &[])
        .with(MovementSystem, "movement_system", &["physics_system"])
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
