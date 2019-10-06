
use amethyst::{
    core::{
        timing::{Time}
    },
    ecs::{Component, System, Join, VecStorage},
    ecs::prelude::{Read, ReadStorage, WriteStorage}
};

use crate::gamestate::{Physics};
use crate::animation::{SpriteAnimation, AnimationType};

pub enum CharacterType {
    Player,
    Enemy
}
impl Component for CharacterType {
    type Storage = VecStorage<Self>;
}
impl Default for CharacterType {
    fn default() -> CharacterType {
        CharacterType::Enemy
    }
}

#[derive(Default, Debug)]
pub struct Player {
    pub is_attacking: bool,
    pub attack_combo: u8,
    pub attack_timer: f32
}
impl Player {
    pub fn new() -> Player {
        Player {
            is_attacking: false,
            attack_combo: 0,
            attack_timer: 0.0
        }
    }
}
impl Component for Player {
    type Storage = VecStorage<Self>;
}

pub struct PlayerSystem;
impl <'a> System<'a> for PlayerSystem {
    type SystemData = (
        ReadStorage<'a, Physics>,
        WriteStorage<'a, Player>,
        WriteStorage<'a, AnimationType>,
        ReadStorage<'a, SpriteAnimation>,
        Read<'a, Time>,
    );
    fn run(&mut self, (physics_set, mut players, mut anim_types, animations, time): Self::SystemData) {
        let dt = time.delta_seconds();
        for (physics, player, anim_type, anim) in (&physics_set, &mut players, &mut anim_types, &animations).join() {
            let mut new_anim_type = anim_type.clone();
            match *anim_type {
                AnimationType::Idle => {
                    if player.is_attacking {
                        let new_combo = (player.attack_combo+1)%3;
                        new_anim_type = AnimationType::Attack(new_combo);
                        player.attack_combo = new_combo;
                    } else if physics.is_jumping {
                        new_anim_type = AnimationType::Jump(false, false);
                    } else if physics.velocity.x.abs() > 0.1 {
                        new_anim_type = AnimationType::Run;
                    }
                },
                AnimationType::Attack(_) => {
                    if anim.finished {
                        new_anim_type = AnimationType::Idle;
                        player.attack_timer = 1.;
                        player.is_attacking = false;
                    }
                },
                AnimationType::Run => {
                    if player.is_attacking {
                        let new_combo = (player.attack_combo+1)%3;
                        new_anim_type = AnimationType::Attack(new_combo);
                        player.attack_combo = new_combo;
                    } else if physics.is_jumping {
                        new_anim_type = AnimationType::Jump(false, true);
                    } else if physics.velocity.x.abs() < 0.1 {
                        new_anim_type = AnimationType::Idle;
                    }
                },
                AnimationType::Jump(falling, running) => {
                    if !physics.is_jumping {
                        new_anim_type = AnimationType::Idle;
                    } else if !falling && physics.velocity.y < 0. {
                        new_anim_type = AnimationType::Jump(true, running);
                    }
                }
            }
            //apply only if changed
            if *anim_type != new_anim_type {
                *anim_type = new_anim_type;
            }

            if player.attack_timer > 0. {
                let new_at = (player.attack_timer - dt).max(0.);
                player.attack_timer = new_at;
            } else if player.attack_combo > 0 {
                player.attack_combo = 0;
            }
        }
    }
}
