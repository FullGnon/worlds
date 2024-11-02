use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{tile::Tile, MapGenerator, MAX_PERLIN_SCALE},
    settings::Settings,
    utils::xy_to_lonlat,
};

pub struct TemperatureGenerator;

impl MapGenerator for TemperatureGenerator {
    fn apply(&self, tile: &mut Tile, x: u32, y: u32, settings: &Settings) {
        let mut value = 0.;
        let mut value_min = -1.;
        let mut value_max = 1.;

        let (lon, lat) = xy_to_lonlat(settings, x, y);

        let scale = settings
            .temperature_gen
            .perlin
            .noise_scale
            .clamp(0., MAX_PERLIN_SCALE);
        let perlin = Perlin::new(settings.temperature_gen.perlin.seed);
        for o in 0..settings.temperature_gen.perlin.octaves {
            let offset_x: f64 = settings.temperature_gen.perlin.offset.x as f64;
            let offset_y: f64 = settings.temperature_gen.perlin.offset.y as f64;
            let frequency: f64 = settings.temperature_gen.perlin.lacunarity.powi(o);
            let amplitude: f64 = settings.temperature_gen.perlin.persistance.powi(o);
            let sample_x = x as f64 / scale * frequency + offset_x;
            let sample_y = y as f64 / scale * frequency + offset_y;

            let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
            value += perlin_value * amplitude;
        }

        let min_lat_factor =
            (-(90_f64.to_radians().cos()) * settings.temperature_gen.scale_lat_factor);
        let lat_factor = (lat.to_radians().cos() * settings.temperature_gen.scale_lat_factor);
        let max_lat_factor = (0_f64.to_radians().cos() * settings.temperature_gen.scale_lat_factor);

        let min_noise_factor = (-1. + 1.) * settings.temperature_gen.noise_factor;
        let noise_factor = (value + 1.) * settings.temperature_gen.noise_factor;
        let max_noise_factor = (1. + 1.) * settings.temperature_gen.noise_factor;

        let min_temperature = min_lat_factor + min_noise_factor - 10.;
        let temperature = lat_factor + noise_factor - 10.;
        let max_temperature = max_lat_factor + max_noise_factor - 10.;

        tile.temperature = temperature;
    }
}
