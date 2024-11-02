use bevy::math::uvec2;
use bevy::prelude::*;
use bevy_fast_tilemap::{bundle::MapBundleManaged, map::Map, plugin::FastTileMapPlugin};
use events::DrawMapEvent;
use generator::get_map_generator;
use noise::{NoiseFn, Perlin};
use shapes::{CircleCenteredShape, ContinentsShape, ShapeGenerator, ShapeGeneratorResource};
pub(crate) mod biomes;
mod events;
mod generator;
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
        let generator = get_map_generator(&config.mode);

        let map = materials.get_mut(map_handle).unwrap();
        let mut m = map.indexer_mut();

        // x0..xN => W - E
        // y0..yN => S - N
        for x in 0..m.size().x {
            for y in 0..m.size().y {
                let tile_index = generator.generate_tile_index(x, y, &config, &texture_tileset);

                m.set(x, y, tile_index);
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
