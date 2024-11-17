use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::settings;
use bevy::render::view::visibility;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{math::uvec2, transform::commands};
use bevy_ecs_tilemap::prelude::*;
use biomes::Biome;
use events::{DrawMapEvent, GenerateMapEvent};
use generator::elevation::{generate as elevation_gen, ElevationGenerator, TileElevation};
use generator::temperature::generate as temperature_gen;
use generator::temperature::{TemperatureGenerator, TileTemperature};
use generator::MapGenerator;
use noise::{NoiseFn, Perlin};
use renderer::elevation::ElevationMapRenderer;
use renderer::temperature::TemperatureMapRenderer;
use shapes::{CircleCenteredShape, ContinentsShape, ShapeGenerator, ShapeGeneratorResource};

use super::settings::{MapMode, Settings};
pub(crate) mod biomes;
mod events;
mod generator;
mod renderer;
mod shapes;

const MAX_PERLIN_SCALE: f64 = 100000.;

#[derive(Component)]
struct LastUpdate(f64);

#[derive(Component)]
struct TemperatureTileMap;

#[derive(Component)]
struct ElevationTileMap;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum MapSet {
    Prepare,
    Generate,
    Render,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum MapState {
    #[default]
    Normal,
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .init_state::<MapState>()
        .configure_sets(
            Update,
            (MapSet::Prepare, MapSet::Generate, MapSet::Render).chain(),
        )
        .add_systems(Startup, setup_map)
        .add_systems(
            Update,
            (
                (elevation_gen, temperature_gen).in_set(MapSet::Generate),
                (update_tiles_color).in_set(MapSet::Render),
            )
                .run_if(resource_changed::<Settings>),
        );
}

fn setup_map(mut commands: Commands, config: Res<Settings>, asset_server: Res<AssetServer>) {
    let map_size = TilemapSize {
        x: config.width,
        y: config.height,
    };

    let tilemap_entity = commands.spawn_empty().id();
    let texture_handle: Handle<Image> = asset_server.load("tiles_white.png");
    let coord_sys: HexCoordSystem = HexCoordSystem::RowEven;

    // Initialize tile storage with empty tiles
    let mut tile_storage = TileStorage::empty(map_size);
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id: TilemapId(tilemap_entity),
                    color: TileColor(Color::srgb(0., 0., 0.)),
                    ..default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize {
        x: config.tile_size.x,
        y: config.tile_size.y,
    };

    let grid_size = tile_size.into();
    let map_type = TilemapType::Hexagon(coord_sys);

    commands.entity(tilemap_entity).insert((
        TilemapBundle {
            grid_size,
            map_type,
            tile_size,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle.clone()),
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            ..default()
        },
        LastUpdate(-1.0),
    ));
}

fn update_tiles_color(
    time: Res<Time>,
    settings: Res<Settings>,
    mut tilemap_query: Query<(&mut Visibility, &mut LastUpdate)>,
    mut tile_query: Query<(&TileElevation, &TileTemperature, &mut TileColor)>,
) {
    let mut elevation_renderer = ElevationMapRenderer;
    let mut temperature_renderer = TemperatureMapRenderer;
    let current_time = time.elapsed_seconds_f64();
    for (mut visibility, mut last_update) in tilemap_query.iter_mut() {
        if current_time - last_update.0 > 0.1 {
            if !settings.elevation && !settings.temperature {
                *visibility = Visibility::Hidden;
            } else {
                *visibility = Visibility::Visible;

                for (tile_elevation, tile_temperature, mut tile_color) in tile_query.iter_mut() {
                    let mut color = Color::srgba(1., 1., 1., 1.);

                    if settings.elevation {
                        color.mix_assign(elevation_renderer.get_color(tile_elevation), 1.);
                    }
                    if settings.temperature {
                        color.mix_assign(
                            temperature_renderer.get_color(tile_temperature, &settings),
                            settings.temperature_factor,
                        );
                    }

                    *tile_color = TileColor(color);
                }
            }
            last_update.0 = current_time;
        }
    }
}
