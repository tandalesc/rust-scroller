
use amethyst::{
    assets::{AssetStorage, Handle, Loader, ProgressCounter, RonFormat},
    prelude::*,
    renderer::{
        formats::texture::ImageFormat,
        Texture,
        sprite::{SpriteSheet, SpriteSheetFormat},
    },
};

use std::fs;
use std::path::Path;
use std::collections::HashMap;

mod main_state;
mod load_map_state;

pub use crate::state::main_state::{
    SCALE_FACTOR,
    GameState
};

pub use crate::state::load_map_state::{
    LoadMapState
};
