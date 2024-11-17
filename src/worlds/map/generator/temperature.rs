use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{MapGenerator, MAX_PERLIN_SCALE},
    settings::Settings,
    utils::xy_to_lonlat,
};

#[derive(Component, Default, Debug)]
pub struct TileTemperature(pub f64);

pub struct TemperatureGenerator;

impl MapGenerator for TemperatureGenerator {
    fn get_value(&self, tile_pos: &TilePos, settings: &Settings) -> f64 {
        let mut value = 0.;

        let (lon, lat) = xy_to_lonlat(settings, tile_pos.x, tile_pos.y);

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
            let sample_x = tile_pos.x as f64 / scale * frequency + offset_x;
            let sample_y = tile_pos.y as f64 / scale * frequency + offset_y;

            let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
            value += perlin_value * amplitude;
        }

        let lat_factor = (lat.to_radians().cos() * settings.temperature_gen.scale_lat_factor);
        let noise_factor = (value + 1.) * settings.temperature_gen.noise_factor;

        lat_factor + noise_factor - 10.
    }

    fn get_min_max(settings: &Settings) -> [f64; 2] {
        let min_lat_factor =
            (-(90_f64.to_radians().cos()) * settings.temperature_gen.scale_lat_factor);
        let max_lat_factor = (0_f64.to_radians().cos() * settings.temperature_gen.scale_lat_factor);

        let min_noise_factor = (-1. + 1.) * settings.temperature_gen.noise_factor;
        let max_noise_factor = (1. + 1.) * settings.temperature_gen.noise_factor;

        let min_temperature = min_lat_factor + min_noise_factor - 10.;
        let max_temperature = max_lat_factor + max_noise_factor - 10.;

        [min_temperature, max_temperature]
    }
}

pub fn generate(
    mut commands: Commands,
    settings: Res<Settings>,
    mut tilemap_query: Query<&TileStorage>,
    tile_query: Query<&TilePos>,
) {
    let generator = TemperatureGenerator;
    for tile_storage in tilemap_query.iter_mut() {
        for tile_pos in tile_query.iter() {
            commands
                .entity(tile_storage.get(tile_pos).unwrap())
                .insert(TileTemperature(generator.get_value(tile_pos, &settings)));
        }
    }
}
