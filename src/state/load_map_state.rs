
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter},
    prelude::*,
    renderer::{
        formats::texture::ImageFormat,
        Texture,
        sprite::{Sprite, SpriteSheet, SpriteSheetFormat},
    },
};

use std::fs;
use std::collections::HashMap;

use crate::state::main_state::{GameState};
use crate::tilemap::{TileMapData};


fn load_sprite_sheet(world: &World, file_name: &str, pc: &mut ProgressCounter) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(format!("{}.png", file_name), ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
        format!("{}.ron", file_name),
        SpriteSheetFormat(texture_handle),
        pc,
        &sprite_sheet_store
    )
}

pub fn load_tile_set(world: &World, file_name: &str, tile_map_data: &TileMapData, pc: &mut ProgressCounter) -> Handle<SpriteSheet> {
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(format!("{}.png", if file_name.ends_with(".tsx") {
            file_name.trim_end_matches(".tsx")
        } else {
            file_name
        }), ImageFormat::default(), (), &texture_storage)
    };
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();

    let (tile_width, tile_height, img_width, img_height) = (
        tile_map_data.tilewidth as u32, tile_map_data.tileheight as u32,
        160, 224
    );
    let sprites_x = img_width/tile_width;
    let sprites_y = img_height/tile_height;
    let sprite_count = (sprites_x*sprites_y) as usize;
    let offsets = [4.; 2]; //origin point is top-left corner
    let mut sprites = Vec::with_capacity(sprite_count);

    //tile maps are just sprites with regularily-spaced sprites
    for y in 0..sprites_y {
        for x in 0..sprites_x {
            let sprite = Sprite::from_pixel_values(
                img_width, img_height,
                tile_width, tile_height,
                x*tile_width, y*tile_height,
                offsets, false, false
            );
            sprites.push(sprite);
        }
    }
    let spritesheet = SpriteSheet {
        texture: texture_handle,
        sprites
    };
    loader.load_from_data(
        spritesheet,
        pc,
        &sprite_sheet_store
    )
}

pub struct LoadMapState {
    tile_map_data: TileMapData,
    tile_set_handles: HashMap<usize, Handle<SpriteSheet>>,
    sprite_handles: HashMap<String, Handle<SpriteSheet>>,
    progress_counters: Vec<ProgressCounter>,
}
impl LoadMapState {
    pub fn new() -> LoadMapState {
        LoadMapState {
            tile_map_data: TileMapData::default(),
            tile_set_handles: HashMap::new(),
            sprite_handles: HashMap::new(),
            progress_counters: Vec::new()
        }
    }
}
impl SimpleState for LoadMapState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let mut world = data.world;
        //load tile map description
        let file = fs::read_to_string("assets/tiled_example.json").unwrap();
        self.tile_map_data = serde_json::from_str(&file).unwrap();
        //load tile map resource handles
        for tileset in &self.tile_map_data.tilesets {
            let mut pc = ProgressCounter::new();
            self.tile_set_handles.insert(tileset.firstgid, load_tile_set(&mut world, &tileset.source, &self.tile_map_data, &mut pc));
            self.progress_counters.push(pc);
        }
        //load sprites
        let (mut pc1, mut pc2) = (ProgressCounter::new(), ProgressCounter::new());
        self.sprite_handles.insert("player_sprite_sheet".to_string(), load_sprite_sheet(&world, "player_sprite_sheet", &mut pc1));
        self.sprite_handles.insert("enemy_kobold_sprite_sheet".to_string(), load_sprite_sheet(&world, "enemy_kobold_sprite_sheet", &mut pc2));
        self.progress_counters.push(pc1);
        self.progress_counters.push(pc2);
    }

    fn update(&mut self, _data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        if self.progress_counters.iter()
            .map(|pc| pc.is_complete())
            .fold(true, |acc,pcs| acc&pcs) {
            Trans::Switch(Box::new(GameState {
                tile_map_data: self.tile_map_data.clone(),
                tile_set_handles: self.tile_set_handles.clone(),
                sprite_handles: self.sprite_handles.clone(),
                map_entities: Vec::new()
            }))
        } else {
            Trans::None
        }
    }
}
