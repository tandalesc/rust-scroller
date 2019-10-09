
use amethyst::{
    assets::{Handle},
    core::{
        transform::{Transform}
    },
    ecs::{Entity},
    prelude::*,
    renderer::{
        Camera,
        SpriteRender, SpriteSheet,
        Transparent
    },
    window::ScreenDimensions
};

use std::collections::HashMap;

use crate::gamestate::{Physics};
use crate::character::{Player, CharacterType};
use crate::animation::{SpriteAnimation, AnimationType, AnimationResource};
use crate::tilemap::{TileMapData, TileMap};

pub const SCALE_FACTOR: f32 = 3.;

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
    sprite_transform.set_translation_xyz(200., 10., 0.);
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
        //let dim = world.read_resource::<ScreenDimensions>();
        //(dim.width(), dim.height())
        (640., 300.)
    };

    let mut transform = Transform::default();
    transform.set_translation_xyz(width/2., height/2., 20.);
    let camera = Camera::standard_2d(width, height);
    world.create_entity()
        .with(camera)
        .with(transform)
        .build();
}

#[derive(Default, Debug)]
pub struct GameState {
    pub tile_map_data: TileMapData,
    pub tile_set_handles: HashMap<usize, Handle<SpriteSheet>>,
    pub sprite_handles: HashMap<String, Handle<SpriteSheet>>,
    pub map_entities: Vec<Entity>,
}
impl SimpleState for GameState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;

        init_player_sprite(&mut world, &self.sprite_handles.get("player_sprite_sheet").unwrap());
        init_enemy_sprite(&mut world, &self.sprite_handles.get("enemy_kobold_sprite_sheet").unwrap());
        init_camera(&mut world);

        let (map_width, map_height, tile_width, tile_height) = (
            self.tile_map_data.width, self.tile_map_data.height,
            self.tile_map_data.tilewidth, self.tile_map_data.tileheight
        );
        let num_layers = self.tile_map_data.layers.len();

        for layer_idx in 0..num_layers {
            let layer = self.tile_map_data.layers.get(layer_idx).unwrap();
            for i in 0..layer.data.len() {
                let tile = *layer.data.get(i).unwrap();
                if tile > 0 {
                    let (x, y) = (
                        (i%map_width*tile_width) as f32, (i/map_width*tile_height) as f32
                    );
                    let mut sprite_transform = Transform::default();
                    sprite_transform.set_translation_xyz(
                        x, (map_height*tile_height) as f32 - y, layer_idx as f32 - 3.
                    );
                    //find greatest map start index that is less than sprite_number
                    let sprite_render = {
                        let map_start_index = self.tile_set_handles.keys()
                            .filter(|&k| k <= &tile)
                            .fold(1, |acc, &k| if k>acc { k } else { acc });
                        SpriteRender {
                            sprite_sheet: self.tile_set_handles.get(&map_start_index).unwrap().clone(),
                            sprite_number: tile - map_start_index
                        }
                    };
                    //create entity in world
                    self.map_entities.push(
                        world.create_entity()
                            .with(sprite_render)
                            .with(sprite_transform)
                            .build()
                    );
                }
            }
        }
        world.add_resource(TileMap::new(self.tile_map_data.clone(), self.tile_set_handles.clone()));
    }
}
