use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{tile::Tile, MapGenerator, MAX_PERLIN_SCALE},
    settings::Settings,
};

pub struct ElevationGenerator;

impl MapGenerator for ElevationGenerator {
    fn apply(&self, tile: &mut Tile, x: u32, y: u32, settings: &Settings) {
        let mut value = 0.;
        let scale = settings
            .elevation_gen
            .noise_scale
            .clamp(0., MAX_PERLIN_SCALE);
        let perlin = Perlin::new(settings.elevation_gen.seed);

        for o in 0..settings.elevation_gen.octaves {
            let offset_x: f64 = settings.elevation_gen.offset.x as f64;
            let offset_y: f64 = settings.elevation_gen.offset.y as f64;
            let frequency: f64 = settings.elevation_gen.lacunarity.powi(o);
            let amplitude: f64 = settings.elevation_gen.persistance.powi(o);
            let sample_x = x as f64 / scale * frequency + offset_x;
            let sample_y = y as f64 / scale * frequency + offset_y;

            let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
            value += perlin_value * amplitude;
        }

        tile.elevation = value;
    }
}
