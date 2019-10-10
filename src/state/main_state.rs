
use amethyst::{
    assets::{Handle},
    core::{
        math as na,
        transform::{Transform}
    },
    ecs::{Entity},
    prelude::*,
    renderer::{
        Camera,
        SpriteRender, SpriteSheet,
        Transparent
    }
};

use std::collections::HashMap;

use crate::system::{Physics, CameraSettings};
use crate::character::{Player, CharacterType};
use crate::animation::{SpriteAnimation, AnimationType, AnimationResource};
use crate::tilemap::{TileMapData, TileMap};

type Vector3 = na::Vector3<f32>;

fn init_player_sprite(world: &mut World, sprite_sheet_handle: &Handle<SpriteSheet>) {
    let mut sprite_transform = Transform::default();
    sprite_transform.set_translation_xyz(30., 64., 0.);
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
    sprite_transform.set_translation_xyz(220., 64., 0.);
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

fn init_camera(world: &mut World, width: f32, height: f32) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(width/2.-8., height/2., 20.);
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

        let (map_width, map_height, tile_width, tile_height) = (
            self.tile_map_data.width, self.tile_map_data.height,
            self.tile_map_data.tilewidth, self.tile_map_data.tileheight
        );
        let pix_map_size = Vector3::new(
            (map_width*tile_width) as f32, (map_height*tile_height) as f32, 0.
        );
        let camera_settings = CameraSettings::new(Vector3::new(30., 30., 20.), pix_map_size);
        init_camera(&mut world, camera_settings.viewport.0, camera_settings.viewport.1);

        let tile_map = TileMap::new(self.tile_map_data.clone(), self.tile_set_handles.clone());
        tile_map.build_map(&mut world);

        world.add_resource(tile_map);
        world.add_resource(camera_settings);

        init_player_sprite(&mut world, &self.sprite_handles.get("player_sprite_sheet").unwrap());
        init_enemy_sprite(&mut world, &self.sprite_handles.get("enemy_kobold_sprite_sheet").unwrap());
    }
}
