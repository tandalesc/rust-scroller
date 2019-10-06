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
        SpriteSheetFormat, Sprite,
        SpriteRender, SpriteSheet,
        Texture, Transparent
    },
    window::ScreenDimensions
};

use crate::character::{Player, CharacterType};
use crate::animation::{SpriteAnimation, AnimationType, AnimationResource};
use crate::tilemap::{TileMapData};
use crate::state::{SCALE_FACTOR};

type Vector3 = na::Vector3<f32>;

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
                physics.velocity.y = 8.;
            }
        }
    }
}
