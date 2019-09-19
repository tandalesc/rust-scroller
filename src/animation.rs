
use amethyst::{
    core::{
        transform::{Transform}
    },
    ecs::{Component, System, Join, VecStorage},
    ecs::prelude::{Read, ReadStorage, WriteStorage},
    renderer::{SpriteRender},
    core::timing::{Time}
};

use super::Physics;

#[derive(Clone, Default, Debug)]
pub struct AnimationData {
    frames: Vec<usize>,
    time_per_frame: f32,
    pub animation_type: AnimationType
}
impl AnimationData {
    pub fn new(frames: Vec<usize>, time_per_frame: f32, animation_type: AnimationType) -> AnimationData {
        AnimationData {
            frames: frames,
            time_per_frame: time_per_frame,
            animation_type: animation_type
        }
    }
}

#[derive(Default, Debug)]
pub struct AnimationResource {
    pub player_idle: AnimationData,
    pub player_run: AnimationData,
}

#[derive(Default, Debug)]
pub struct SpriteAnimation {
    frames: Vec<usize>,
    current_frame: usize,
    time_per_frame: f32,
    elapsed_time: f32,
    pub animation_type: AnimationType
}
impl SpriteAnimation {
    pub fn from_data(animation_details: AnimationData) -> SpriteAnimation {
        SpriteAnimation::new(animation_details.frames, animation_details.time_per_frame, animation_details.animation_type)
    }
    pub fn new(frames: Vec<usize>, time_per_frame: f32, animation_type: AnimationType) -> SpriteAnimation {
        SpriteAnimation {
            frames: frames,
            current_frame: 0,
            time_per_frame: time_per_frame,
            elapsed_time: 0.0,
            animation_type: animation_type
        }
    }
    pub fn get_frame(&self) -> usize {
        self.frames[self.current_frame]
    }
}
impl Component for SpriteAnimation {
    type Storage = VecStorage<Self>;
}

#[derive(Debug, Clone)]
pub enum AnimationType {
    Idle,
    Run,
    Unset
}
impl Default for AnimationType {
    fn default() -> Self { AnimationType::Unset }
}

pub struct AnimationSystem;
impl <'a> System<'a> for AnimationSystem {
    type SystemData = (
        ReadStorage<'a, Physics>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, SpriteAnimation>,
        Read<'a, AnimationResource>,
        Read<'a, Time>,
    );
    fn run(&mut self, (physics_set, mut transforms, mut sprite_renders, mut animations, animation_resource, time): Self::SystemData) {
        for (physics, transform, sprite_render, anim) in (&physics_set, &mut transforms, &mut sprite_renders, &mut animations).join() {
            match anim.animation_type {
                AnimationType::Idle => {
                    if physics.velocity.x.abs() > 0.1 {
                        *anim = SpriteAnimation::from_data(animation_resource.player_run.clone());
                    }
                },
                AnimationType::Run => {
                    if physics.velocity.x.abs() < 0.1 {
                        *anim = SpriteAnimation::from_data(animation_resource.player_idle.clone());
                    }
                },
                _ => { /* Do nothing */ }
            };
            //rotate sprite depending on direction we're facing
            if physics.velocity.x > 0.1 {
                transform.set_rotation_y_axis(0.);
            } else if physics.velocity.x < -0.1 {
                transform.set_rotation_y_axis(std::f32::consts::PI);
            }
            //progress each animation to the next frame
            anim.elapsed_time += time.delta_seconds();
            let frame_count = (anim.elapsed_time / anim.time_per_frame) as usize % anim.frames.len();
            if frame_count != anim.current_frame {
                anim.current_frame = frame_count;
                sprite_render.sprite_number = anim.get_frame();
            }
        }
    }
}
