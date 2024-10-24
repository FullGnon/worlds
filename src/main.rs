use std::{fs, thread::sleep, time::Duration};

use bevy::reflect::Reflect;
use bevy::{
    ecs::{query, reflect},
    math::uvec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    utils::HashMap,
};
use bevy_fast_tilemap::prelude::*;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin, DefaultInspectorConfigPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use serde::Deserialize;

mod biomes;

use biomes::Biome;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanCamPlugin,
            EguiPlugin,
            FastTileMapPlugin::default(),
        ))
        .add_plugins(DefaultInspectorConfigPlugin)
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        .add_systems(Startup, (setup))
        .add_systems(Update, redraw_map)
        .observe(on_draw_map)
        .run();
}

fn scale(value: f64, min: f64, max: f64, scale_min: f64, scale_max: f64) -> f64 {
    ((value - min) / (max - min)) * (scale_max - scale_min) + scale_min
}

fn scale_to_index(value: f64, min: f64, max: f64, scale_min: f64, scale_max: f64) -> usize {
    scale(value, min, max, scale_min, scale_max).round() as usize
}

#[derive(Reflect, Resource, InspectorOptions)]
struct Configuration {
    height: u32,
    width: u32,
    tile_size: Vec2,

    seed: u32,
    noise_scale: f64,
    octaves: i32,
    lacunarity: f64,
    persistance: f64,

    biomes: HashMap<String, Biome>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            height: 100,
            width: 100,
            tile_size: Vec2::new(16., 16.),
            seed: random(),
            noise_scale: 100.,
            octaves: 3,
            lacunarity: 2.,
            persistance: 0.5,
            biomes: load_biomes("assets/biomes"),
        }
    }
}

#[derive(Component)]
struct Tile;

#[derive(Event)]
struct DrawMapEvent;

fn on_draw_map(
    trigger: Trigger<DrawMapEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    config: Res<Configuration>,
) {
    let perlin = Perlin::new(config.seed);

    let tiles_texture = asset_server.load("temperate_forest/temperate_forest.png");

    let map = Map::builder(
        // Map size
        uvec2(config.width, config.height),
        // Tile atlas
        tiles_texture,
        // Tile Size
        config.tile_size,
    )
    .build_and_initialize(|m| {
        for x in 0..m.size().x {
            for y in 0..m.size().y {
                let mut noise_value = 0.;
                let scale = if config.noise_scale <= 0.0 {
                    0.0
                } else {
                    config.noise_scale
                };
                for o in 0..config.octaves {
                    let frequency: f64 = config.lacunarity.powi(o);
                    let amplitude: f64 = config.persistance.powi(o);
                    // TODO add octave offset (but why ?)
                    let sample_x = x as f64 / scale * frequency;
                    let sample_y = y as f64 / scale * frequency;

                    let perlin_value = perlin.get([sample_x, sample_y, 0.0]);

                    noise_value += perlin_value * amplitude;
                }

                let index =
                    scale_to_index(noise_value, -1., 1., 0., config.colors.len() as f64 - 1.)
                        .clamp(0, config.colors.len() - 1);

                /*let color = Color::srgb(
                    config.colors[0][0] as f32 / 255.,
                    config.colors[0][1] as f32 / 255.,
                    config.colors[0][2] as f32 / 255.,
                );*/

                m.set(x, y, index as u32);
            }
        }
    });

    commands.spawn(MapBundleManaged {
        material: materials.add(map),
        ..default()
    });
}

fn setup(mut commands: Commands, config: Res<Configuration>) {
    commands
        .spawn(Camera2dBundle {
            /*transform: Transform::from_xyz(
                (config.width as f32 * config.tile_size.x) / 2.,
                (config.height as f32 * config.tile_size.y) / 2.,
                0.0,
            ),*/
            ..default()
        })
        .insert(PanCam::default());

    commands.trigger(DrawMapEvent);
}

fn redraw_map(
    mut commands: Commands,
    config: Res<Configuration>,
    query_tiles: Query<Entity, With<Tile>>,
) {
    if config.is_changed() {
        for entity in query_tiles.iter() {
            commands.entity(entity).despawn();
        }

        commands.trigger(DrawMapEvent);
    }
}
