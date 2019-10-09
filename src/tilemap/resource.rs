
use amethyst::{
    assets::{Handle},
    core::{
        transform::{Transform}
    },
    prelude::*,
    renderer::{
        SpriteRender
    },
    renderer::{SpriteSheet}
};

use std::collections::HashMap;

use super::TileMapData;

#[derive(Default, Debug)]
pub struct TileMap {
    pub tile_map_data: TileMapData,
    tile_set_handles: HashMap<usize, Handle<SpriteSheet>>,
    pub collidable_layer: usize
}
impl TileMap {
    pub fn new(tile_map_data: TileMapData, tile_set_handles: HashMap<usize, Handle<SpriteSheet>>) -> TileMap {
        //find which layer is marked collidable
        let mut collidable_layer_idx = 0;
        for layer_idx in 0..tile_map_data.layers.len() {
            let layer = tile_map_data.layers.get(layer_idx).unwrap();
            if layer.name == "collidable" {
                collidable_layer_idx = layer_idx;
                break;
            }
        }
        TileMap {
            collidable_layer: collidable_layer_idx,
            tile_map_data,
            tile_set_handles
        }
    }
    pub fn build_map(&self, world: &mut World) {
        let num_layers = self.tile_map_data.layers.len();
        let (map_width, map_height, tile_width, tile_height) = (
            self.tile_map_data.width, self.tile_map_data.height,
            self.tile_map_data.tilewidth, self.tile_map_data.tileheight
        );
        for layer_idx in 0..num_layers {
            let layer = self.tile_map_data.layers.get(layer_idx).unwrap();
            for i in 0..layer.data.len() {
                let tile = *layer.data.get(i).unwrap();
                if tile > 0 {
                    let (x, y) = (
                        (i%map_width*tile_width) as f32, (i/map_width*tile_height) as f32
                    );
                    let z = if layer.name.contains("background") {
                        -1.
                    } else if layer.name.contains("foreground") {
                        1.
                    } else {
                        0.
                    };
                    let mut sprite_transform = Transform::default();
                    sprite_transform.set_translation_xyz(
                        x, (map_height*tile_height) as f32 - y, z
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
                    world.create_entity()
                        .with(sprite_render)
                        .with(sprite_transform)
                        .build();
                }
            }
        }
    }
    pub fn is_tile_collidable(&self, i: usize) -> bool {
        let collidable_layer = self.tile_map_data.layers.get(self.collidable_layer).unwrap();
        self.is_valid_position(i) && *collidable_layer.data.get(i).unwrap() > 0
    }
    pub fn is_valid_position(&self, i: usize) -> bool {
        i < self.tile_map_data.width * self.tile_map_data.height
    }
    pub fn xy_to_i(&self, x: usize, y: usize) -> usize {
        self.tile_map_data.width * (self.tile_map_data.height - y) + x
    }
    pub fn pix_to_map(&self, pix: f32) -> usize {
        (pix / self.tile_map_data.tilewidth as f32).floor() as usize
    }
    pub fn map_to_pix(&self, map: usize) -> f32 {
        (map * self.tile_map_data.tilewidth) as f32
    }
 }

pub struct Tile {
    tile_index: usize,
    tile_type: TileType
}

pub enum TileType {
    Background,
    Collidable
}
