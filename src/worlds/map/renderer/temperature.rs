use crate::worlds::map::tile::Tile;
use crate::worlds::map::tileset::TextureTileSet;
use crate::worlds::utils::scale_to_index;

use super::MapRenderer;

pub struct TemperatureMapRenderer {
    start: usize,
    size: usize,
}

impl TemperatureMapRenderer {
    pub fn new(texture_tileset: &TextureTileSet) -> Self {
        let elevation_index = texture_tileset.biomes_mapping["Temperature"];
        let (start, size) = texture_tileset.biomes_position[elevation_index as usize].into();

        Self {
            start: start as usize,
            size: size as usize,
        }
    }
}

impl MapRenderer for TemperatureMapRenderer {
    fn get_tile_index(&self, tile: &Tile) -> u32 {
        let mut index = 0;

        scale_to_index(
            tile.temperature,
            -1.,
            1.,
            self.start as f64,
            (self.start - self.size) as f64,
        )
    }
}
