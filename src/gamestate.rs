use amethyst::{
    assets::{AssetStorage, Loader, Handle},
    core::{
        math as na,
        timing::{Time},
        transform::{Transform}
    },
    ecs::{Entity, Component, System, Join, VecStorage, NullStorage},
    ecs::prelude::{
        Read,
        ReadStorage,
        WriteStorage,
        Resources
    },
    input::{InputHandler, StringBindings},
    prelude::*,
    renderer::{
        Camera,
        ImageFormat,
        SpriteSheetFormat,
        SpriteRender, SpriteSheet,
        Texture, Transparent
    },
    window::ScreenDimensions
};

use crate::character::{Player, CharacterType};
use crate::animation::{SpriteAnimation, AnimationType, AnimationResource};

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
    let char_type = CharacterType::Player;
    let anim_type = AnimationType::Idle;
    let animation_data =  world.read_resource::<AnimationResource>().data(&char_type, &anim_type);
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(Physics::default())
        .with(Player::new())
        .with(char_type)
        .with(anim_type)
        .with(Transparent)
        .build();
}

fn init_enemy_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(200., 30., 0.);
    let sprite_render = SpriteRender {
        sprite_sheet: sprite_sheet_handle.clone(),
        sprite_number: 0
    };
    let char_type = CharacterType::Enemy;
    let anim_type = AnimationType::Attack(0);
    let animation_data =  world.read_resource::<AnimationResource>().data(&char_type, &anim_type);
    let animation = SpriteAnimation::from_data(animation_data);
    world.create_entity()
        .with(sprite_render)
        .with(sprite_transform)
        .with(animation)
        .with(Physics::default())
        .with(char_type)
        .with(anim_type)
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
pub struct GameState;
impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;
        let player_sprite_sheet_handle = load_sprite_sheet(&world, "player_sprite_sheet");
        let enemy_kobold_sprite_sheet_handle = load_sprite_sheet(&world, "enemy_kobold_sprite_sheet");
        init_player_sprite(&mut world, &player_sprite_sheet_handle);
        init_enemy_sprite(&mut world, &enemy_kobold_sprite_sheet_handle);
        init_camera(&mut world);
    }
}

#[derive(Debug)]
pub struct Physics {
    pub acceleration: Vector3,
    pub velocity: Vector3,
    pub mass: f32,
    pub friction: f32,
    pub jump_cooldown: u32,
    pub is_jumping: bool
}
impl Component for Physics {
    type Storage = VecStorage<Self>;
}
impl Default for Physics {
    fn default() -> Physics {
        Physics {
            acceleration: Vector3::new(0., 0., 0.),
            velocity: Vector3::new(0., 0., 0.),
            mass: 1.,
            friction: 0.5,
            jump_cooldown: 0,
            is_jumping: false
        }
    }
}

pub struct PhysicsSystem;
impl <'a> System<'a> for PhysicsSystem {
    type SystemData = (
        WriteStorage<'a, Physics>,
        WriteStorage<'a, Transform>,
        Read<'a, Time>
    );
    fn run(&mut self, (mut physics_set, mut transform, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (physics, transform) in (&mut physics_set, &mut transform).join() {
            if transform.translation().y > 20. || physics.velocity.y > 0. {
                physics.acceleration.y = -9.8;
                physics.velocity.y += physics.acceleration.y*dt;
            } else if physics.is_jumping {
                //we hit the ground, reset jumping flag
                physics.is_jumping = false;
                physics.jump_cooldown = 5; //5 frames
                physics.acceleration.y = 0.;
                physics.velocity.y = 0.;
            }
            if physics.jump_cooldown > 0 {
                physics.jump_cooldown -= 1;
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

pub struct MovementSystem;
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
            physics.acceleration.x = cx as f32 * 12.;
            if attack && !player.is_attacking  {
                player.is_attacking = true;
            } else if jump && !physics.is_jumping && !player.is_attacking && physics.jump_cooldown == 0 {
                physics.is_jumping = true;
                physics.velocity.y = 4.;
            }
        }
    }
}
