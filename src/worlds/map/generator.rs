use crate::worlds::settings::Settings;

use super::tile::Tile;

pub mod elevation;
pub mod shape;
pub mod temperature;

pub trait MapGenerator: Send + Sync {
    fn apply(&self, tile: &mut Tile, x: u32, y: u32, settings: &Settings);
}
