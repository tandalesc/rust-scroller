
use amethyst::{
    core::{
        math as na,
    }
};

use crate::tilemap::TileMap;

type Point3 = na::Point3<f32>;
type Vector3 = na::Vector3<f32>;

#[derive(Debug)]
pub struct Hitbox {
    pub position: Point3,
    pub size: Vector3
}
impl Hitbox {
    pub fn collides_with(&self, other: &Hitbox) -> bool {
        self.position.x < other.position.x+other.size.x &&
        self.position.x+self.size.x > other.position.x &&
        self.position.y < other.position.y+other.size.y &&
        self.position.y+self.size.y > other.position.y
    }
    pub fn colliding_with_ceiling(&self, map: &TileMap) -> bool {
        let num_iter_x = (self.size.x/map.tile_map_data.tilewidth as f32).ceil() as usize;
        for x in 0..num_iter_x/2 {
            let pix_x = self.position.x + (x as f32)*map.tile_map_data.tilewidth as f32 + self.size.x/4.;
            let pix_y = self.position.y + self.size.y;
            if map.is_tile_collidable(map.xy_to_i(map.pix_to_map(pix_x), map.pix_to_map(pix_y))) {
                return true;
            }
        }
        return false;
    }
    pub fn colliding_with_ground(&self, map: &TileMap) -> bool {
        let num_iter_x = (self.size.x/map.tile_map_data.tilewidth as f32).ceil() as usize;
        for x in 0..num_iter_x {
            let pix_x = self.position.x + (x as f32)*map.tile_map_data.tilewidth as f32;
            let pix_y = self.position.y - self.size.y;
            if map.is_tile_collidable(map.xy_to_i(map.pix_to_map(pix_x), map.pix_to_map(pix_y))) {
                return true;
            }
        }
        return false;
    }
    pub fn colliding_with_left_wall(&self, map: &TileMap) -> bool {
        let num_iter_y = (self.size.y/map.tile_map_data.tileheight as f32).ceil() as usize;
        for y in 0..num_iter_y/2 {
            let pix_x = self.position.x;
            let pix_y = self.position.y - (y as f32)*map.tile_map_data.tileheight as f32 - self.size.y/4.;
            if map.is_tile_collidable(map.xy_to_i(map.pix_to_map(pix_x), map.pix_to_map(pix_y))) {
                return true;
            }
        }
        return false;
    }
    pub fn colliding_with_right_wall(&self, map: &TileMap) -> bool {
        let num_iter_y = (self.size.y/map.tile_map_data.tileheight as f32).ceil() as usize;
        for y in 0..num_iter_y/2 {
            let pix_x = self.position.x + self.size.x;
            let pix_y = self.position.y - (y as f32)*map.tile_map_data.tileheight as f32 - self.size.y/4.;
            if map.is_tile_collidable(map.xy_to_i(map.pix_to_map(pix_x), map.pix_to_map(pix_y))) {
                return true;
            }
        }
        return false;
    }
}

#[derive(Debug)]
pub struct Collision {
    pub direction: Vector3
}
