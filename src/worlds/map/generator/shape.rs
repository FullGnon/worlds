use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{shapes::ShapeGenerator, tileset::TextureTileSet, MAX_PERLIN_SCALE},
    settings::Settings,
    utils::scale_to_index,
};

use super::MapGenerator;

pub struct ElevationShapeGenerator;

impl MapGenerator for ElevationShapeGenerator {
    #[allow(clippy::borrowed_box)]
    fn generate_tile_index(
        &self,
        x: u32,
        y: u32,
        config: &Settings,
        texture_tileset: &TextureTileSet,
    ) -> u32 {
        let biome_index = texture_tileset.biomes_mapping["Elevation"];
        let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index as usize].into();

        //let value = shape_generator.generate(x, y, config);

        let value = 0.;

        scale_to_index(
            value,
            0.,
            1.,
            min_index as f64,
            (min_index + n_tiles) as f64 - 1.,
        )
        .clamp(min_index, min_index + n_tiles - 1)
    }
}
