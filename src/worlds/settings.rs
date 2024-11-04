use bevy::prelude::*;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::InspectorOptions;
use rand::{random, Rng};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::worlds::map::biomes::Biome;

use super::map::biomes::load_biomes;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Settings>()
        .register_type::<Settings>()
        .add_plugins(ResourceInspectorPlugin::<Settings>::new());
}

// TBD: Condition the use of InspectorOptions
#[derive(Reflect, Resource, InspectorOptions)]
pub struct Settings {
    pub height: u32,
    pub width: u32,
    pub tile_size: Vec2,

    pub mode: MapMode,
    pub world_shape: WorldShapeGeneration,
    pub shaped_world: bool,

    pub elevation_gen: PerlinConfiguration,
    pub temperature_gen: TemperatureGeneration,
    pub sea_level: f64,

    pub biomes: HashMap<String, Biome>,
}

impl Default for Settings {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            height: 100,
            width: 100,
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
            mode: MapMode::Elevation,
            world_shape: WorldShapeGeneration::default(),
            shaped_world: true,
        }
    }
}

#[derive(Reflect)]
pub enum MapMode {
    Elevation,
    Temperature,
    WorldShapeMode,
}

#[derive(Reflect)]
pub struct TemperatureGeneration {
    pub perlin: PerlinConfiguration,
    pub scale_lat_factor: f64,
    pub noise_factor: f64,
}

#[derive(Reflect)]
pub enum WorldShapeEnum {
    CenteredShape,
    Continents,
}

#[derive(Reflect)]
pub struct WorldShapeGeneration {
    pub shape: WorldShapeEnum,
    pub shape_factor: f64,
    pub shape_radius: f64,
    pub count_continent: usize,
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
pub struct PerlinConfiguration {
    pub seed: u32,
    pub noise_scale: f64,
    pub octaves: i32,
    pub lacunarity: f64,
    pub persistance: f64,
    pub offset: Vec2,
}
