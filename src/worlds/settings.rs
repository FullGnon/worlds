use bevy::prelude::*;
use bevy_inspector_egui::InspectorOptions;
use rand::{random, Rng};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Settings>().register_type::<Settings>();
}

// TBD: Condition the use of InspectorOptions
#[derive(Reflect, Resource, InspectorOptions)]
struct Settings {
    height: u32,
    width: u32,
    tile_size: Vec2,

    mode: MapMode,
    world_shape: WorldShapeGeneration,
    shaped_world: bool,

    elevation_gen: PerlinConfiguration,
    temperature_gen: TemperatureGeneration,
    sea_level: f64,
}

impl Default for Settings {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            height: 400,
            width: 400,
            tile_size: Vec2::new(16., 16.),
            elevation_gen: PerlinConfiguration {
                seed: random(),
                noise_scale: 100.,
                octaves: 4,
                lacunarity: 2.5,
                persistance: 0.5,
                offset: Vec2::new(
                    rng.gen_range(-100000..100000) as f32,
                    rng.gen_range(-100000..100000) as f32,
                ),
            },
            temperature_gen: TemperatureGeneration {
                perlin: PerlinConfiguration {
                    seed: random(),
                    noise_scale: 200.,
                    octaves: 3,
                    lacunarity: 4.,
                    persistance: 0.3,
                    offset: Vec2::new(
                        rng.gen_range(-100000..100000) as f32,
                        rng.gen_range(-100000..100000) as f32,
                    ),
                },
                scale_lat_factor: 40.,
                noise_factor: 20.,
            },
            sea_level: 0.05,
            // TBD: It could be better to load biomes as asset directly
            biomes: load_biomes(Path::new("assets/biomes")).unwrap(),
            mode: MapMode::WorldShapeMode,
            world_shape: WorldShapeGeneration::default(),
            shaped_world: true,
        }
    }
}

#[derive(Reflect)]
enum MapMode {
    Elevation,
    Temperature,
    WorldShapeMode,
}

#[derive(Reflect)]
struct TemperatureGeneration {
    perlin: PerlinConfiguration,
    scale_lat_factor: f64,
    noise_factor: f64,
}

#[derive(Reflect)]
enum WorldShapeEnum {
    CenteredShape,
    Continents,
}

#[derive(Reflect)]
struct WorldShapeGeneration {
    shape: WorldShapeEnum,
    shape_factor: f64,
    shape_radius: f64,
    count_continent: usize,
}

impl Default for WorldShapeGeneration {
    fn default() -> Self {
        Self {
            shape: WorldShapeEnum::Continents,
            shape_factor: 1.1,
            shape_radius: 200.,
            count_continent: 2,
        }
    }
}

#[derive(Reflect)]
struct PerlinConfiguration {
    seed: u32,
    noise_scale: f64,
    octaves: i32,
    lacunarity: f64,
    persistance: f64,
    offset: Vec2,
}
