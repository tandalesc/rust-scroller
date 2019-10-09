
use amethyst::{
    assets::{Handle},
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
                println!("FOUND COLLIDABLE LAYER");
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
