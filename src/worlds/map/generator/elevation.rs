use bevy::{prelude::*, text::scale_value};
use bevy_ecs_tilemap::prelude::*;
use noise::{NoiseFn, Perlin};

use crate::worlds::{
    map::{DrawMapEvent, GenerateMapEvent, MapGenerator, MAX_PERLIN_SCALE},
    settings::Settings,
    utils::scale,
};

#[derive(Component, Default, Debug)]
pub struct TileElevation(pub f64);

pub struct ElevationGenerator;

impl MapGenerator for ElevationGenerator {
    fn get_value(&self, tile_pos: &TilePos, settings: &Settings) -> f64 {
        let mut value = 0.;
        let noise_scale = settings
            .elevation_gen
            .noise_scale
            .clamp(0., MAX_PERLIN_SCALE);
        let perlin = Perlin::new(settings.elevation_gen.seed);

        for o in 0..settings.elevation_gen.octaves {
            let offset_x: f64 = settings.elevation_gen.offset.x as f64;
            let offset_y: f64 = settings.elevation_gen.offset.y as f64;
            let frequency: f64 = settings.elevation_gen.lacunarity.powi(o);
            let amplitude: f64 = settings.elevation_gen.persistance.powi(o);
            let sample_x = tile_pos.x as f64 / noise_scale * frequency + offset_x;
            let sample_y = tile_pos.y as f64 / noise_scale * frequency + offset_y;

            let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
            value += perlin_value * amplitude;
        }

        scale(value, -1., 1., -20., 20.).clamp(-20., 20.)
    }
    fn get_min_max(settings: &Settings) -> [f64; 2] {
        [-20., 20.]
    }
}

pub fn generate(
    mut commands: Commands,
    config: Res<Settings>,
    mut tilemap_query: Query<&TileStorage>,
    tile_query: Query<&TilePos>,
) {
    let generator = ElevationGenerator;
    for tile_storage in tilemap_query.iter_mut() {
        for tile_pos in tile_query.iter() {
            commands
                .entity(tile_storage.get(tile_pos).unwrap())
                .insert(TileElevation(generator.get_value(tile_pos, &config)));
        }
    }
}
