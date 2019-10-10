use amethyst::{
    core::{
        math as na,
        timing::{Time},
        transform::{Transform}
    },
    ecs::{Component, System, Join, DenseVecStorage},
    ecs::prelude::{
        Read,
        Write,
        ReadStorage,
        WriteStorage
    },
    renderer::{
        Camera
    },
    input::{InputHandler, StringBindings},
};

use crate::character::{Player, CharacterType};
use crate::hitbox::{Hitbox};
use crate::tilemap::{TileMap};

type Point3 = na::Point3<f32>;
type Vector3 = na::Vector3<f32>;

#[derive(Debug)]
pub struct CameraSettings {
    pub boundaries: Vector3,
    pub target: Vector3,
    pub velocity: Vector3,
    pub viewport: (f32, f32)
}
impl CameraSettings {
    pub fn new(target: Vector3, boundaries: Vector3) -> CameraSettings {
        CameraSettings {
            target,
            boundaries,
            velocity: Vector3::new(0., 0., 0.),
            viewport: (400., 300.)
        }
    }
}
impl Default for CameraSettings {
    fn default() -> CameraSettings {
        CameraSettings::new(Vector3::new(0.,0.,0.), Vector3::new(800.,600.,0.))
    }
}

pub struct UpdateCameraSystem;
impl <'a> System<'a> for UpdateCameraSystem {
    type SystemData = (
        ReadStorage<'a, Camera>,
        WriteStorage<'a, Transform>,
        Read<'a, CameraSettings>,
        Read<'a, Time>,
    );
    fn run(&mut self, (cameras, mut transform_set, camera_settings, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (camera, transform) in (&cameras, &mut transform_set).join() {
            let disp = (camera_settings.target - transform.translation())*dt;
            if disp.norm() > 0. {
                let mut translate = transform.translation().clone() + Vector3::new(disp.x, disp.y, 0.);
                translate.x = translate.x.min(camera_settings.boundaries.x - camera_settings.viewport.0/2.).max(camera_settings.viewport.0/2.);
                translate.y = translate.y.min(camera_settings.boundaries.y - camera_settings.viewport.1/2.).max(camera_settings.viewport.1/2.);
                transform.set_translation(translate);
            }
        }
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
        ReadStorage<'a, CharacterType>,
        WriteStorage<'a, Physics>,
        WriteStorage<'a, Transform>,
        Write<'a, CameraSettings>,
        Read<'a, TileMap>,
        Read<'a, Time>,
    );
    fn run(&mut self, (character_types, mut physics_set, mut transform, mut camera_settings, tilemap, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (character_type, physics, transform) in (&character_types, &mut physics_set, &mut transform).join() {
            let trans = transform.translation();
            let hb = Hitbox {
                position: Point3::new(trans.x, trans.y, 0.),
                size: Vector3::new(14., 10., 0.)
            };
            let new_trans = transform.translation() + physics.velocity*dt + physics.acceleration*dt*dt/2.;
            let new_hb = Hitbox {
                position: Point3::new(new_trans.x, new_trans.y, 0.),
                size: Vector3::new(14., 10., 0.)
            };
            let dist = trans - new_trans;
            let total_hb = {
                let tl_hb = if trans.x <= new_trans.x && trans.y <= new_trans.y { hb } else { new_hb };
                Hitbox {
                    position: Point3::new(tl_hb.position.x, tl_hb.position.y, 0.),
                    size: Vector3::new(dist.x.abs() + tl_hb.size.x, dist.y.abs() + tl_hb.size.y, 0.)
                }
            };

            let (left, top, right, bottom) = (
                total_hb.colliding_with_left_wall(&tilemap),
                total_hb.colliding_with_ceiling(&tilemap),
                total_hb.colliding_with_right_wall(&tilemap),
                total_hb.colliding_with_ground(&tilemap),
            );

            if bottom {
                if physics.velocity.y < 0. {
                    physics.acceleration.y = 0.;
                    physics.velocity.y = -dist.y;
                    if physics.is_jumping {
                        physics.is_jumping = false;
                        physics.jump_cooldown = 5; //5 frames
                    }
                }
            } else {
                if !physics.is_jumping {
                    physics.is_jumping = true;
                }
                physics.acceleration.y = -9.8;
            }
            if left && physics.acceleration.x < 0. {
                physics.acceleration.x = 0.;
                physics.velocity.x = -dist.x;
            }
            if right && physics.acceleration.x > 0. {
                physics.acceleration.x = 0.;
                physics.velocity.x = -dist.x;
            }
            if top && physics.velocity.y > 0. {
                physics.velocity.y = -dist.y;
            }

            if physics.jump_cooldown > 0 {
                physics.jump_cooldown -= 1;
            }
            physics.velocity += physics.acceleration*dt;
            physics.velocity.x -= physics.velocity.x*physics.friction*10.*dt;
            //clamp translation so new coordinates are always on the screen
            let mut new_translation = transform.translation() + physics.velocity;
            new_translation.x = new_translation.x.max(0.);
            new_translation.y = new_translation.y.max(0.);

            transform.set_translation(new_translation);
            //target player with camera after updating position
            if let CharacterType::Player = character_type {
                camera_settings.target = new_translation;
            }
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
                physics.velocity.y = 4.;
            }
        }
    }
}
