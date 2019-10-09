
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
    pub fn get_map_collisions(&self, map: &TileMap) -> Vec<Collision> {
        let mut collisions = Vec::new();
        let num_iter_x = (self.size.x/map.tile_map_data.tilewidth as f32).ceil() as usize;
        let num_iter_y = (self.size.y/map.tile_map_data.tileheight as f32).ceil() as usize;
        for x in 0..num_iter_x+2 {
            let pix_x = self.position.x - (x as f32 - 1.)*map.tile_map_data.tilewidth as f32;
            for y in 0..num_iter_y+2 {
                let pix_y = self.position.y - (y as f32 - 1.)*map.tile_map_data.tileheight as f32;
                if map.is_tile_collidable(map.xy_to_i(map.pix_to_map(pix_x), map.pix_to_map(pix_y))) {
                    let other_hb = Hitbox {
                        position: Point3::new(pix_x.floor(), pix_y.floor(), 0.),
                        size: Vector3::new(map.tile_map_data.tilewidth as f32, map.tile_map_data.tileheight as f32, 0.)
                    };
                    if self.collides_with(&other_hb) {
                        let direction = other_hb.position - (self.position-self.size/2.);
                        collisions.push(Collision { direction: Vector3::new(direction.x, direction.y, 0.) });
                    }
                }
            }
        }
        collisions
    }
}

#[derive(Debug)]
pub struct Collision {
    pub direction: Vector3
}
