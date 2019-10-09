use amethyst::{
    core::{
        math as na,
        timing::{Time},
        transform::{Transform}
    },
    ecs::{Component, System, Join, DenseVecStorage},
    ecs::prelude::{
        Read,
        WriteStorage
    },
    input::{InputHandler, StringBindings},
};

use crate::character::{Player};
use crate::state::{SCALE_FACTOR};
use crate::hitbox::{Hitbox};
use crate::tilemap::{TileMap};

type Point3 = na::Point3<f32>;
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
    type Storage = DenseVecStorage<Self>;
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
        Read<'a, TileMap>,
        Read<'a, Time>,
    );
    fn run(&mut self, (mut physics_set, mut transform, tilemap, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (physics, transform) in (&mut physics_set, &mut transform).join() {
            let trans = transform.translation();
            let hb = Hitbox {
                position: Point3::new(trans.x, trans.y-8., 0.),
                size: Vector3::new(8., 16., 0.)
            };
            let map_collisions = hb.get_map_collisions(&tilemap);
            let num_collisions = map_collisions.len();
            let avg_collision = map_collisions.iter()
                .fold(Vector3::new(0.,0.,0.), |acc, col| acc + col.direction/(num_collisions as f32));

            if avg_collision.y <= 0. || physics.velocity.y > 0. {
                physics.acceleration.y = -9.8;
            } else if avg_collision.y > 0. {
                //we hit the ground, reset jumping flag
                if physics.is_jumping {
                    physics.is_jumping = false;
                    physics.jump_cooldown = 5; //5 frames
                }
                physics.acceleration.y = 0.;
                physics.velocity.y = 0.;
            }

            if physics.jump_cooldown > 0 {
                physics.jump_cooldown -= 1;
            }
            physics.velocity += physics.acceleration*dt;
            physics.velocity.x -= physics.velocity.x*physics.friction*9.8*dt;
            //clamp translation so new coordinates are always on the screen
            let mut new_translation = transform.translation() + physics.velocity;
            for collision in &map_collisions {
                if collision.direction.x.abs() > 0. && collision.direction.y == 0. {
                    new_translation += collision.direction*dt;
                } else {
                    new_translation.y += collision.direction.y*dt;
                }
            }
            new_translation.x = new_translation.x.max(0.);
            new_translation.y = new_translation.y.max(0.);
            transform.set_translation(new_translation);
        }
    }
}

pub struct MovementSystem;
impl <'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, Player>,
        WriteStorage<'a, Physics>,
        Read<'a, InputHandler<StringBindings>>
    );
    fn run(&mut self, (mut players, mut physics_set, input): Self::SystemData) {
        let (cx, _cy, attack, jump) = (
            input.axis_value("x").unwrap(),
            input.axis_value("y").unwrap(),
            input.action_is_down("attack").unwrap(),
            input.action_is_down("jump").unwrap()
        );
        for (player, physics) in (&mut players, &mut physics_set).join() {
            physics.acceleration.x = cx as f32 * 8.;
            if attack && !player.is_attacking  {
                player.is_attacking = true;
            } else if jump && !physics.is_jumping && !player.is_attacking && physics.jump_cooldown == 0 {
                physics.is_jumping = true;
                physics.velocity.y = 5.;
            }
        }
    }
}
