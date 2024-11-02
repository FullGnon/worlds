use bevy::math::uvec2;
use bevy::prelude::*;
use bevy_fast_tilemap::{bundle::MapBundleManaged, map::Map, plugin::FastTileMapPlugin};
use events::DrawMapEvent;
use noise::{NoiseFn, Perlin};
use shapes::{CircleCenteredShape, ContinentsShape, ShapeGenerator, ShapeGeneratorResource};
pub(crate) mod biomes;
mod events;
mod shapes;
mod tileset;

use super::{
    settings::{MapMode, Settings, WorldShapeEnum},
    utils::{scale_to_index, xy_to_lonlat},
};
use tileset::TextureTileSet;

const MAX_PERLIN_SCALE: f64 = 100000.;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(FastTileMapPlugin::default())
        .add_plugins((biomes::plugin, tileset::plugin, shapes::plugin))
        .add_systems(Startup, setup_map)
        .add_systems(Update, update_map)
        .observe(on_draw_map);
}

fn setup_map(
    mut commands: Commands,
    config: Res<Settings>,
    texture_tileset: Res<TextureTileSet>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
    mut shape_generator_resource: ResMut<ShapeGeneratorResource>,
) {
    let tiles_texture = asset_server.load(texture_tileset.path.clone());

    let map = Map::builder(
        // Map size
        uvec2(config.width, config.height),
        // Tile atlas
        tiles_texture,
        // Tile Size
        config.tile_size,
    )
    .build_and_set(|_| 2);

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });

    shape_generator_resource.generator.init(&config);

    commands.trigger(DrawMapEvent);
}

fn on_draw_map(
    trigger: Trigger<DrawMapEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_tileset: Res<TextureTileSet>,
    mut materials: ResMut<Assets<Map>>,
    config: Res<Settings>,
    maps: Query<&Handle<Map>>,
    shape_generator: Res<ShapeGeneratorResource>,
) {
    for map_handle in maps.iter() {
        let map = materials.get_mut(map_handle).unwrap();
        let mut m = map.indexer_mut();

        // x0..xN => W - E
        // y0..yN => S - N
        for x in 0..m.size().x {
            for y in 0..m.size().y {
                let tile_index = match config.mode {
                    MapMode::Elevation => get_elevation_tile_index(
                        x,
                        y,
                        &config,
                        &texture_tileset,
                        &shape_generator.generator,
                    ),
                    MapMode::Temperature => {
                        get_temperature_tile_index(x, y, &config, &texture_tileset)
                    }
                    MapMode::WorldShapeMode => get_world_shape_tile_index(
                        x,
                        y,
                        &config,
                        &texture_tileset,
                        &shape_generator.generator,
                    ),
                };

                m.set(x, y, tile_index as u32);
            }
        }
    }
}

fn update_map(
    mut commands: Commands,
    config: Res<Settings>,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
    mut shape_generator_resource: ResMut<ShapeGeneratorResource>,
) {
    if config.is_changed() {
        match config.world_shape.shape {
            WorldShapeEnum::CenteredShape => {
                shape_generator_resource.generator = Box::new(CircleCenteredShape);
            }
            WorldShapeEnum::Continents => {
                shape_generator_resource.generator = Box::new(ContinentsShape::default());
            }
        }
        shape_generator_resource.generator.init(&config);
        commands.trigger(DrawMapEvent);
    }
}

#[allow(clippy::borrowed_box)]
fn get_world_shape_tile_index(
    x: u32,
    y: u32,
    config: &Settings,
    texture_tileset: &TextureTileSet,
    shape_generator: &Box<dyn ShapeGenerator>,
) -> usize {
    let biome_index = texture_tileset.biomes_mapping["Elevation"];
    let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index].into();

    let value = shape_generator.generate(x, y, config);

    scale_to_index(
        value,
        0.,
        1.,
        min_index as f64,
        (min_index + n_tiles) as f64 - 1.,
    )
    .clamp(min_index, min_index + n_tiles - 1)
}

fn get_temperature_tile_index(
    x: u32,
    y: u32,
    config: &Settings,
    texture_tileset: &TextureTileSet,
) -> usize {
    let mut value = 0.;
    let mut value_min = -1.;
    let mut value_max = 1.;

    let (lon, lat) = xy_to_lonlat(config, x, y);

    let biome_index = texture_tileset.biomes_mapping["Temperature"];
    let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index].into();

    let scale = config
        .temperature_gen
        .perlin
        .noise_scale
        .clamp(0., MAX_PERLIN_SCALE);
    let perlin = Perlin::new(config.temperature_gen.perlin.seed);
    for o in 0..config.temperature_gen.perlin.octaves {
        let offset_x: f64 = config.temperature_gen.perlin.offset.x as f64;
        let offset_y: f64 = config.temperature_gen.perlin.offset.y as f64;
        let frequency: f64 = config.temperature_gen.perlin.lacunarity.powi(o);
        let amplitude: f64 = config.temperature_gen.perlin.persistance.powi(o);
        let sample_x = x as f64 / scale * frequency + offset_x;
        let sample_y = y as f64 / scale * frequency + offset_y;

        let perlin_value = perlin.get([sample_x, sample_y, 0.0]);
        value += perlin_value * amplitude;
    }

    let min_lat_factor = (-(90_f64.to_radians().cos()) * config.temperature_gen.scale_lat_factor);
    let lat_factor = (lat.to_radians().cos() * config.temperature_gen.scale_lat_factor);
    let max_lat_factor = (0_f64.to_radians().cos() * config.temperature_gen.scale_lat_factor);

    let min_noise_factor = (-1. + 1.) * config.temperature_gen.noise_factor;
    let noise_factor = (value + 1.) * config.temperature_gen.noise_factor;
    let max_noise_factor = (1. + 1.) * config.temperature_gen.noise_factor;

    let min_temperature = min_lat_factor + min_noise_factor - 10.;
    let temperature = lat_factor + noise_factor - 10.;
    let max_temperature = max_lat_factor + max_noise_factor - 10.;

    // Select biome tile
    scale_to_index(
        temperature,
        min_temperature,
        max_temperature,
        min_index as f64,
        (min_index + n_tiles) as f64 - 1.,
    )
    .clamp(min_index, min_index + n_tiles - 1)
}

#[allow(clippy::borrowed_box)]
fn get_elevation_tile_index(
    x: u32,
    y: u32,
    config: &Settings,
    texture_tileset: &TextureTileSet,
    shape_generator: &Box<dyn ShapeGenerator>,
) -> usize {
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
    let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index].into();

    if config.shaped_world {
        let shape_value = shape_generator.generate(x, y, config);
        value -= shape_value * config.world_shape.shape_factor;
    }

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
