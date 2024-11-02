use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{shapes::ShapeGenerator, tileset::TextureTileSet, MAX_PERLIN_SCALE},
    settings::Settings,
    utils::scale_to_index,
};

use super::MapGenerator;

pub struct ElevationGenerator;

impl MapGenerator for ElevationGenerator {
    #[allow(clippy::borrowed_box)]
    fn generate_tile_index(
        &self,
        x: u32,
        y: u32,
        config: &Settings,
        texture_tileset: &TextureTileSet,
    ) -> u32 {
        let mut value = 0.;
        let scale = config.elevation_gen.noise_scale.clamp(0., MAX_PERLIN_SCALE);
        let perlin = Perlin::new(config.elevation_gen.seed);

        for o in 0..config.elevation_gen.octaves {
            let offset_x: f64 = config.elevation_gen.offset.x as f64;
            let offset_y: f64 = config.elevation_gen.offset.y as f64;
            let frequency: f64 = config.elevation_gen.lacunarity.powi(o);
            let amplitude: f64 = config.elevation_gen.persistance.powi(o);
            let sample_x = x as f64 / scale * frequency + offset_x;
            let sample_y = y as f64 / scale * frequency + offset_y;

            let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
            value += perlin_value * amplitude;
        }

        let mut value_min = -1.;
        let mut value_max = 1.;

        // Select biome
        let biome_index = texture_tileset.biomes_mapping["Elevation"];
        let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index as usize].into();

        /*if config.shaped_world {
            let shape_value = shape_generator.generate(x, y, config);
            value -= shape_value * config.world_shape.shape_factor;
        }*/

        // Select biome tile
        scale_to_index(
            value.clamp(value_min, value_max),
            value_min,
            value_max,
            min_index as f64,
            (min_index + n_tiles) as f64 - 1.,
        )
        .clamp(min_index, min_index + n_tiles - 1)
    }
}
