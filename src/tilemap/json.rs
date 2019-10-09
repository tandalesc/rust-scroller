
use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileMapData {
    pub width: usize,
    pub height: usize,
    infinite: bool,
    pub layers: Vec<TileLayerData>,
    nextlayerid: usize,
    nextobjectid: usize,
    orientation: String,
    renderorder: String,
    tiledversion: String,
    pub tilewidth: usize,
    pub tileheight: usize,
    pub tilesets: Vec<TileSetSource>,
    r#type: String,
    version: f32
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileLayerData {
    pub data: Vec<usize>,
    pub width: usize,
    pub height: usize,
    pub id: usize,
    pub name: String,
    r#type: String,
    opacity: f32,
    visible: bool,
    x: usize,
    y: usize
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TileSetSource {
    pub firstgid: usize,
    pub source: String
}
