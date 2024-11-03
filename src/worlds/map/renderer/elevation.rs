use crate::worlds::map::tile::Tile;
use crate::worlds::map::tileset::TextureTileSet;
use crate::worlds::utils::scale_to_index;

use super::MapRenderer;

pub struct ElevationMapRenderer {
    start: usize,
    size: usize,
}

impl ElevationMapRenderer {
    pub fn new(texture_tileset: &TextureTileSet) -> Self {
        let index = texture_tileset.biomes_mapping["Elevation"];
        let (start, size) = texture_tileset.biomes_position[index as usize].into();

        Self {
            start: start as usize,
            size: size as usize,
        }
    }
}

impl MapRenderer for ElevationMapRenderer {
    fn get_tile_index(&self, tile: &Tile) -> u32 {
        let mut index = 0;

        scale_to_index(
            tile.elevation,
            -1.,
            1.,
            self.start as f64,
            self.start as f64 - self.size as f64,
        )
    }
}
