use super::tile::Tile;

pub mod elevation;
pub mod temperature;

pub trait MapRenderer: Send + Sync {
    fn get_tile_index(&self, tile: &Tile) -> u32;
}
