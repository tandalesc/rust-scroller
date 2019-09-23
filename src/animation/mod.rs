
use amethyst::{
    core::{
        transform::{Transform}
    },
    ecs::{Component, System, Join, VecStorage},
    ecs::prelude::{Read, ReadStorage, WriteStorage},
    renderer::{SpriteRender},
    core::timing::{Time}
};

use crate::character::{CharacterType};
use crate::gamestate::{Physics};

#[derive(Clone, Default, Debug)]
pub struct AnimationData {
    frames: Vec<usize>,
    time_per_frame: f32,
    loop_anim: bool,
    pub animation_type: AnimationType
}
impl AnimationData {
    pub fn new(frames: Vec<usize>, time_per_frame: f32, animation_type: AnimationType, passed_loop: bool) -> AnimationData {
        AnimationData {
            frames: frames,
            time_per_frame: time_per_frame,
            animation_type: animation_type,
            loop_anim: passed_loop
        }
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct AnimationResource;
impl AnimationResource {
    pub fn data(&self, char_type: &CharacterType, anim_type: &AnimationType) -> AnimationData {
        match char_type {
            //default implementation - temp for enemy
            CharacterType::Player => { match anim_type {
                AnimationType::Idle => AnimationData::new(vec![0,1,2,3], 1./6., AnimationType::Idle, true),
                AnimationType::Run => AnimationData::new(vec![4,5,6,7,8,9], 1./10., AnimationType::Run, true),
                AnimationType::Jump(falling, running) => { match running {
                    true => AnimationData::new(vec![31,32,33,34], 1./10., AnimationType::Jump(*falling, true), true),
                    false => match falling {
                        false => AnimationData::new(vec![10,11,12,13], 1./10., AnimationType::Jump(false, *running), false),
                        true => AnimationData::new(vec![13,12,11], 1./10., AnimationType::Jump(true, *running), false)
                    }
                } },
                AnimationType::Attack(combo) => { match combo {
                    1 => AnimationData::new(vec![19,20,21,22,23,24], 1./10., AnimationType::Attack(1), true),
                    2 => AnimationData::new(vec![25,26,27,28,29,30], 1./10., AnimationType::Attack(2), true),
                    _ => AnimationData::new(vec![14,15,16,17,18], 1./10., AnimationType::Attack(*combo), true)
                } },
                _ => AnimationData::new(vec![0,1,2,3], 1./6., anim_type.clone(), true), //default to idle animation
            } },
            CharacterType::Enemy => { match anim_type {
                AnimationType::Idle => AnimationData::new(vec![0,1,2,3], 1./6., AnimationType::Idle, true),
                AnimationType::Run => AnimationData::new(vec![4,5,6,7,8,9], 1./10., AnimationType::Run, true),
                AnimationType::Attack(combo) => AnimationData::new(vec![10,11,12,13,14], 1./10., AnimationType::Attack(*combo), true),
                _ => AnimationData::new(vec![0,1,2,3], 1./6., anim_type.clone(), true) //default to idle animation
            } }
        }
    }
}

#[derive(Default, Debug)]
pub struct SpriteAnimation {
    frames: Vec<usize>,
    current_frame: usize,
    time_per_frame: f32,
    elapsed_time: f32,
    loop_anim: bool,
    pub finished: bool,
    pub animation_type: AnimationType
}
impl SpriteAnimation {
    pub fn from_data(animation_details: AnimationData) -> SpriteAnimation {
        SpriteAnimation::new(
            animation_details.frames,
            animation_details.time_per_frame,
            animation_details.animation_type,
            animation_details.loop_anim
        )
    }
    pub fn new(frames: Vec<usize>, time_per_frame: f32, animation_type: AnimationType, passed_loop: bool) -> SpriteAnimation {
        SpriteAnimation {
            frames: frames,
            current_frame: 0,
            time_per_frame: time_per_frame,
            elapsed_time: 0.0,
            loop_anim: passed_loop,
            finished: false,
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

#[derive(Debug, Clone, PartialEq)]
pub enum AnimationType {
    Idle,
    Run,
    Jump(bool, bool),
    Attack(u8)
}
impl Default for AnimationType {
    fn default() -> Self { AnimationType::Idle }
}
impl Component for AnimationType {
    type Storage = VecStorage<Self>;
}

pub struct AnimationSystem;
impl <'a> System<'a> for AnimationSystem {
    type SystemData = (
        ReadStorage<'a, Physics>,
        ReadStorage<'a, CharacterType>,
        WriteStorage<'a, Transform>,
        WriteStorage<'a, SpriteRender>,
        WriteStorage<'a, SpriteAnimation>,
        ReadStorage<'a, AnimationType>,
        Read<'a, AnimationResource>,
        Read<'a, Time>,
    );
    fn run(&mut self, (physics_set, character_types, mut transforms, mut sprite_renders, mut animations, anim_types, animation_resource, time): Self::SystemData) {
        for (physics, char_type, transform, sprite_render, anim, anim_type) in (&physics_set, &character_types, &mut transforms, &mut sprite_renders, &mut animations, &anim_types).join() {
            //create a new sprite object only if the animation has changed
            if anim.animation_type != *anim_type {
                *anim = SpriteAnimation::from_data(animation_resource.data(char_type, anim_type));
            }
            //rotate sprite depending on direction we're facing
            if physics.velocity.x > 0.1 {
                transform.set_rotation_y_axis(0.);
            } else if physics.velocity.x < -0.1 {
                transform.set_rotation_y_axis(std::f32::consts::PI);
            }
            //progress each animation to the next frame
            anim.elapsed_time += time.delta_seconds();
            let frame_count = if anim.loop_anim {
                //use modulus to keep looping through valid range
                (anim.elapsed_time / anim.time_per_frame) as usize % anim.frames.len()
            } else {
                //don't loop animation, just stop at the last frame
                ((anim.elapsed_time / anim.time_per_frame) as usize).min(anim.frames.len()-1)
            };
            if frame_count != anim.current_frame {
                if !anim.finished && frame_count == anim.frames.len()-1 {
                    anim.finished = true;
                }
                anim.current_frame = frame_count;
                sprite_render.sprite_number = anim.get_frame();
            }
        }
    }
}
