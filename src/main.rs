use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::{fs, thread::sleep, time::Duration};

use bevy::app::DynEq;
use bevy::input::mouse::MouseWheel;
use bevy::log::tracing_subscriber::util::SubscriberInitExt;
use bevy::reflect::Reflect;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::texture::{CompressedImageFormats, TextureError};
use bevy::window::{PresentMode, WindowTheme};
use bevy::{
    ecs::{query, reflect},
    math::uvec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_fast_tilemap::prelude::*;
use bevy_inspector_egui::bevy_egui;
use bevy_inspector_egui::{
    bevy_egui::EguiPlugin, prelude::*, quick::ResourceInspectorPlugin, DefaultInspectorConfigPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use image::{GenericImage, ImageEncoder, ImageFormat, Pixel, PixelWithColorType, Rgb, RgbImage};
use noise::{NoiseFn, Perlin};
use rand::prelude::*;
use serde::Deserialize;

mod biomes;

use biomes::{load_biomes, Biome};
use tempfile::{tempfile, Builder};

const MAX_PERLIN_SCALE: f64 = 10000.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Worlds".into(),
                name: Some("worlds".into()),
                resolution: (1600., 1000.).into(),
                present_mode: PresentMode::AutoVsync,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins((PanCamPlugin, EguiPlugin, FastTileMapPlugin::default()))
        .add_plugins(DefaultInspectorConfigPlugin)
        .register_type::<HashMap<String, Biome>>()
        .init_resource::<Configuration>()
        .init_resource::<ShapeGeneratorResource>()
        .register_type::<Configuration>()
        .add_plugins(ResourceInspectorPlugin::<Configuration>::new())
        .init_resource::<TextureTileSet>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update_map))
        .add_systems(
            PreUpdate,
            (absorb_egui_inputs.after(bevy_egui::systems::process_input_system),),
        )
        .observe(on_draw_map)
        .run();
}

fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_wheel: ResMut<Events<MouseWheel>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    if !(ctx.wants_pointer_input() || ctx.is_pointer_over_area()) {
        return;
    }
    let modifiers = [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ];

    let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));

    mouse.reset_all();
    mouse_wheel.clear();
    keyboard.reset_all();

    for key in pressed.into_iter().flatten() {
        keyboard.press(key);
    }
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

    mode: MapMode,
    world_shape: WorldShapeGeneration,
    shaped_world: bool,

    elevation_gen: PerlinConfiguration,
    temperature_gen: TemperatureGeneration,
    sea_level: f64,

    #[reflect(ignore)]
    biomes: HashMap<String, Biome>,
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

// This resource should be bound to a state ?
#[derive(Resource)]
struct ShapeGeneratorResource {
    generator: Box<dyn ShapeGenerator>,
}

impl Default for ShapeGeneratorResource {
    fn default() -> Self {
        Self {
            generator: Box::new(CircleCenteredShape),
        }
    }
}

trait ShapeGenerator: Send + Sync {
    fn init(&mut self, config: &Configuration);
    fn generate(&self, x: u32, y: u32, config: &Configuration) -> f64;
}

#[derive(Default)]
struct CircleCenteredShape;

impl ShapeGenerator for CircleCenteredShape {
    fn init(&mut self, config: &Configuration) {}
    fn generate(&self, x: u32, y: u32, config: &Configuration) -> f64 {
        let center_x = config.width as f64 / 2.;
        let center_y = config.height as f64 / 2.;

        let distance = ((x as f64 - center_x).powi(2) + (y as f64 - center_y).powi(2)).sqrt();
        let distance_max = (center_x.powi(2) + center_y.powi(2)).sqrt();

        scale(distance, 0., distance_max, -1., 1.)
    }
}

struct ContinentsShape {
    count: usize,
    random_points: Vec<(f64, f64)>,
    seed: u32,
}

impl Default for ContinentsShape {
    fn default() -> Self {
        Self {
            count: 1,
            random_points: Vec::new(),
            seed: 0,
        }
    }
}

impl ShapeGenerator for ContinentsShape {
    fn init(&mut self, config: &Configuration) {
        let x_max = config.width as f64;
        let y_max = config.height as f64;
        let mut rng = rand::thread_rng();

        self.count = config.world_shape.count_continent;
        self.seed = config.elevation_gen.seed;
        self.random_points.clear();

        for i in 0..self.count {
            let x = rng.gen_range(0_f64..=x_max);
            let y = rng.gen_range(0_f64..=y_max);

            self.random_points.push((x, y));
        }

        println!("RANDOM POINTS {:?}", self.random_points);
        let distance_max = ((config.width.pow(2) + config.height.pow(2)) as f64).sqrt();
        println!("DISTANCE MAX {}", distance_max);
    }

    fn generate(&self, x: u32, y: u32, config: &Configuration) -> f64 {
        let distance_max = ((config.width.pow(2) + config.height.pow(2)) as f64).sqrt();
        let mut distance = distance_max;

        for i in 0..self.count {
            let (point_x, point_y) = self.random_points[i];
            let point_distance =
                ((x as f64 - point_x).powi(2) + (y as f64 - point_y).powi(2)).sqrt();
            if point_distance < distance {
                distance = point_distance;
            }
        }

        scale(distance, 0., config.world_shape.shape_radius, -1., 1.)
    }
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

impl Default for Configuration {
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
            biomes: load_biomes(Path::new("assets/biomes")).unwrap(),
            mode: MapMode::WorldShapeMode,
            world_shape: WorldShapeGeneration::default(),
            shaped_world: true,
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
    texture_tileset: Res<TextureTileSet>,
    mut materials: ResMut<Assets<Map>>,
    config: Res<Configuration>,
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

#[allow(clippy::borrowed_box)]
fn get_world_shape_tile_index(
    x: u32,
    y: u32,
    config: &Configuration,
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

fn xy_to_lonlat(config: &Configuration, x: u32, y: u32) -> (f64, f64) {
    let x_min = 0_f64;
    let y_min = 0_f64;
    let x_max = (config.width - 1) as f64;
    let y_max = (config.height - 1) as f64;

    let lon = scale(x as f64, x_min, x_max, -180., 180.);
    let lat = scale(y as f64, y_min, y_max, -90., 90.);

    (lon, lat)
}

fn lonlat_to_xy(config: &Configuration, lon: f64, lat: f64) -> (u32, u32) {
    let x_min = 0_f64;
    let y_min = 0_f64;
    let x_max = (config.width - 1) as f64;
    let y_max = (config.height - 1) as f64;
    let x = scale(lon, -180., 180., x_min, x_max);
    let y = scale(lat, -90., 90., y_min, y_max);

    (x as u32, y as u32)
}

fn get_temperature_tile_index(
    x: u32,
    y: u32,
    config: &Configuration,
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
    config: &Configuration,
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

#[derive(Debug)]
struct TextureTile {
    index: usize,
    biome_name: String,
    tile_name: String,
}

#[derive(Resource, Debug)]
struct TextureTileSet {
    path: PathBuf,
    tileset: Vec<TextureTile>,
    biomes_position: Vec<[usize; 2]>,
    biomes_mapping: HashMap<String, usize>,
}

impl FromWorld for TextureTileSet {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Configuration>();

        let enabled_biomes = config
            .biomes
            .clone()
            .into_iter()
            .filter(|(_, biome)| biome.enabled.unwrap_or_default())
            .collect::<HashMap<String, Biome>>();

        build_tiles_texture_from_biomes(&enabled_biomes).unwrap()
    }
}

fn setup(
    mut commands: Commands,
    config: Res<Configuration>,
    texture_tileset: Res<TextureTileSet>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
    mut shape_generator_resource: ResMut<ShapeGeneratorResource>,
) {
    let camera_transform = Transform {
        scale: Vec3::splat(6.0),
        translation: Vec3::new(-1200., 0., 0.),
        ..default()
    };
    commands
        .spawn(Camera2dBundle {
            transform: camera_transform,
            ..default()
        })
        .insert(PanCam { ..default() });

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

fn build_tiles_texture_from_biomes(
    biomes: &HashMap<String, Biome>,
) -> Result<TextureTileSet, Box<dyn Error>> {
    // TODO !! Hardcoded number of tiles !!
    let n_tile: u32 = 50;
    let tile_size: u32 = 16;
    let width: u32 = n_tile * tile_size;
    let mut img_buffer = RgbImage::new(width, tile_size);

    let mut idx_tile: usize = 0;
    let mut biomes_position = Vec::new();
    let mut tileset: Vec<TextureTile> = Vec::new();
    let mut biome_current_index = 0_usize;
    let mut biomes_mapping: HashMap<String, usize> = HashMap::new();
    for (biome_name, biome) in biomes.iter() {
        if biome.tiles.is_none() {
            continue;
        }

        let biome_tiles_clone = biome.tiles.clone().unwrap();
        let mut biome_tiles: Vec<(&String, &[u8; 3])> = biome_tiles_clone.iter().collect();
        biome_tiles.sort();

        for (tile_name, &tile_color) in biome_tiles.iter() {
            for x in 0..tile_size {
                for y in 0..tile_size {
                    img_buffer.put_pixel(x + (tile_size * idx_tile as u32), y, Rgb(tile_color));
                }
            }
            let tiletexture = TextureTile {
                index: idx_tile,
                biome_name: biome_name.clone(),
                tile_name: tile_name.to_string(),
            };
            tileset.push(tiletexture);
            idx_tile += 1;
        }
        biomes_position.push([biome_current_index, idx_tile - biome_current_index]);
        biomes_mapping.insert(biome_name.clone(), biomes_position.len() - 1);
        biome_current_index = idx_tile;
    }

    // Create a temporary file
    let mut path = env::temp_dir().join("world_tilesets.png");
    let mut file = File::create(&path)?;

    // Bind the writer to the opened file
    let mut writer = BufWriter::new(file);

    // Write bytes into the file as PNG format
    img_buffer.write_to(&mut writer, ImageFormat::Png);

    Ok(TextureTileSet {
        path,
        tileset,
        biomes_position,
        biomes_mapping,
    })
}

fn update_map(
    mut commands: Commands,
    config: Res<Configuration>,
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

#[derive(Resource)]
struct DayTimer {
    timer: Timer,
}

impl Default for DayTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.1, TimerMode::Repeating),
        }
    }
}

fn tick_day_timer(mut day_timer: ResMut<DayTimer>, time: Res<Time>) {
    day_timer.timer.tick(time.delta());
}

fn asteroids_fly(day_timer: Res<DayTimer>, mut config: ResMut<Configuration>) {
    if day_timer.timer.finished() {
        config.temperature_gen.perlin.offset.x += 0.05;
    }
}
