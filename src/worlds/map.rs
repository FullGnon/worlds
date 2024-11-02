use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::{math::uvec2, transform::commands};
use bevy_fast_tilemap::{bundle::MapBundleManaged, map::Map, plugin::FastTileMapPlugin};
use biomes::Biome;
use events::{DrawMapEvent, GenerateMapEvent};
use generator::elevation::ElevationGenerator;
use generator::MapGenerator;
use noise::{NoiseFn, Perlin};
use renderer::elevation::ElevationMapRenderer;
use renderer::MapRenderer;
use shapes::{CircleCenteredShape, ContinentsShape, ShapeGenerator, ShapeGeneratorResource};
use tile::{Tile, TileMatrixResource};
pub(crate) mod biomes;
mod events;
mod generator;
mod renderer;
mod shapes;
mod tile;
mod tileset;

use super::{
    settings::{MapMode, Settings, WorldShapeEnum},
    utils::{scale_to_index, xy_to_lonlat},
};
use tileset::{TextureTile, TextureTileSet};

const MAX_PERLIN_SCALE: f64 = 100000.;

#[derive(Resource)]
pub struct MapGeneratorsResource {
    generators: Vec<Box<dyn MapGenerator>>,
}

impl FromWorld for MapGeneratorsResource {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Settings>();

        let mut generators: Vec<Box<dyn MapGenerator>> = Vec::new();

        // TODO: to build from settings
        generators.push(Box::new(ElevationGenerator));

        Self { generators }
    }
}

#[derive(Resource)]
struct MapRendererResource {
    renderer: Box<dyn MapRenderer>,
}

impl FromWorld for MapRendererResource {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Settings>();
        let texture_tileset = world.resource::<TextureTileSet>();

        // TODO: to build from settings
        Self {
            renderer: Box::new(ElevationMapRenderer::new(texture_tileset)),
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<TextureTileSet>()
        .init_resource::<TileMatrixResource>()
        .init_resource::<MapGeneratorsResource>()
        .init_resource::<MapRendererResource>()
        .add_plugins(FastTileMapPlugin::default())
        .add_plugins((biomes::plugin, shapes::plugin))
        .add_systems(Startup, setup_map)
        .add_systems(Update, update_map)
        .observe(on_generate_map)
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

    commands.trigger(GenerateMapEvent);
}

fn on_generate_map(
    trigger: Trigger<GenerateMapEvent>,
    mut commands: Commands,
    mut tile_matrix: ResMut<TileMatrixResource>,
    config: Res<Settings>,
    mut generators: ResMut<MapGeneratorsResource>,
) {
    println!("GENERATE MAP");
    for x in 0..config.width {
        for y in 0..config.height {
            let mut tile: Tile = Tile::default();

            for generator in &generators.generators {
                generator.apply(&mut tile, x, y, &config);
            }

            tile_matrix.set(x as usize, y as usize, tile);
        }
    }

    commands.trigger(DrawMapEvent);
}

fn on_draw_map(
    trigger: Trigger<DrawMapEvent>,
    asset_server: Res<AssetServer>,
    texture_tileset: Res<TextureTileSet>,
    mut materials: ResMut<Assets<Map>>,
    config: Res<Settings>,
    maps: Query<&Handle<Map>>,
    tile_matrix: Res<TileMatrixResource>,
    renderer: Res<MapRendererResource>,
) {
    println!("RENDER MAP");
    for map_handle in maps.iter() {
        let map = materials.get_mut(map_handle).unwrap();
        let mut m = map.indexer_mut();

        // x0..xN => W - E
        // y0..yN => S - N
        for x in 0..tile_matrix.width {
            for y in 0..tile_matrix.height {
                let tile = tile_matrix.get(x, y).unwrap();
                let tile_index = renderer.renderer.get_tile_index(tile);

                m.set(x.try_into().unwrap(), y.try_into().unwrap(), tile_index);
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
        commands.trigger(GenerateMapEvent);
    }
}
