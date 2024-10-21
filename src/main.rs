use std::{thread::sleep, time::Duration};

use bevy::{
    ecs::{query, reflect},
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin, DefaultInspectorConfigPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin, EguiPlugin))
        .add_plugins(DefaultInspectorConfigPlugin)
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_plugins(ResourceInspectorPlugin::<Configuration>::default())
        .add_systems(Startup, (setup))
        .add_systems(Update, redraw_map)
        .observe(on_draw_map)
        .run();
}

fn scale(value: f32, min: f32, max: f32, scale_min: f32, scale_max: f32) -> f32 {
    ((value - min) / (max - min)) * (scale_max - scale_min) + scale_min
}

fn scale_to_index(value: f32, min: f32, max: f32, scale_min: f32, scale_max: f32) -> usize {
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

    colors: Vec<[usize; 3]>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            height: 100,
            width: 100,
            tile_size: Vec2::new(8., 8.),
            seed: random(),
            noise_scale: 100.,
            octaves: 3,
            lacunarity: 2.,
            persistance: 0.8,
            colors: [
                [35, 30, 50], // Ocean
                [61, 75, 100],
                [126, 148, 162],
                [188, 170, 108], // Sand
                [178, 183, 160], // Grass
                [147, 161, 135],
                [81, 80, 49],
                [81, 80, 49], // Mountains
                [163, 151, 135],
                [71, 64, 59],
                [255, 255, 255], // Snow
            ]
            .to_vec(),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    config: Res<Configuration>,
) {
    let perlin = Perlin::new(config.seed);
    let shape = Mesh2dHandle(meshes.add(Rectangle::new(config.tile_size.x, config.tile_size.y)));

    for x in 0..config.width {
        for y in 0..config.height {
            let mut noise_value: f32 = 0.;
            let scale = if config.noise_scale <= 0.0 {
                0.0
            } else {
                config.noise_scale
            };
            for o in 0..config.octaves {
                let frequency: f64 = config.lacunarity.powi(o);
                let amplitude: f64 = config.persistance.powi(o);
                let sample_x = x as f64 / scale * frequency;
                let sample_y = y as f64 / scale * frequency;

                let perlin_value = perlin.get([sample_x, sample_y, 0.0]);

                noise_value += (perlin_value * amplitude) as f32;
            }

            let index = scale_to_index(noise_value, -1., 1., 0., config.colors.len() as f32 - 1.)
                .clamp(0, config.colors.len() - 1);
            let pos_x = x as f32 * config.tile_size.x;
            let pos_y = y as f32 * config.tile_size.y;
            let color = Color::srgb(
                config.colors[index][0] as f32 / 255.,
                config.colors[index][1] as f32 / 255.,
                config.colors[index][2] as f32 / 255.,
            );

            commands.spawn((
                MaterialMesh2dBundle {
                    mesh: shape.clone(),
                    material: materials.add(color),
                    transform: Transform::from_xyz(pos_x, pos_y, 0.0),
                    ..default()
                },
                Tile,
            ));
        }
    }
}

fn setup(mut commands: Commands, config: Res<Configuration>) {
    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(
            (config.width as f32 * config.tile_size.x) / 2.,
            (config.height as f32 * config.tile_size.y) / 2.,
            0.0,
        ),
        ..default()
    });

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
