use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::hash::Hash;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::{fs, thread::sleep, time::Duration};

use bevy::input::mouse::MouseWheel;
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
        .init_resource::<Configuration>()
        .register_type::<Configuration>()
        .add_plugins(ResourceInspectorPlugin::<Configuration>::new())
        .init_resource::<TextureTileSet>()
        .add_systems(Startup, setup)
        .add_systems(Update, update_map)
        .add_systems(
            PreUpdate,
            absorb_egui_inputs.after(bevy_egui::systems::process_input_system),
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

    elevation_gen: PerlinConfiguration,
    biome_gen: PerlinConfiguration,
    sea_level: f64,

    biomes: HashMap<String, Biome>,
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
            biome_gen: PerlinConfiguration {
                seed: random(),
                noise_scale: 100.,
                octaves: 3,
                lacunarity: 1.,
                persistance: 0.7,
                offset: Vec2::new(
                    rng.gen_range(-100000..100000) as f32,
                    rng.gen_range(-100000..100000) as f32,
                ),
            },
            sea_level: 0.05,
            biomes: load_biomes(Path::new("assets/biomes")).unwrap(),
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
) {
    for map_handle in maps.iter() {
        let map = materials.get_mut(map_handle).unwrap();
        let mut m = map.indexer_mut();

        let perlin_elevation = Perlin::new(config.elevation_gen.seed);
        let perlin_biome = Perlin::new(config.biome_gen.seed);

        // x0..xN => W - E
        // y0..yN => S - N
        for x in 0..m.size().x {
            for y in 0..m.size().y {
                let mut elevation_value = 0.;
                let elevation_scale = if config.elevation_gen.noise_scale <= 0.0 {
                    0.0
                } else {
                    config.elevation_gen.noise_scale
                };

                for o in 0..config.elevation_gen.octaves {
                    let offset_x: f64 = config.elevation_gen.offset.x as f64;
                    let offset_y: f64 = config.elevation_gen.offset.y as f64;
                    let frequency: f64 = config.elevation_gen.lacunarity.powi(o);
                    let amplitude: f64 = config.elevation_gen.persistance.powi(o);
                    let sample_x = x as f64 / elevation_scale * frequency + offset_x;
                    let sample_y = y as f64 / elevation_scale * frequency + offset_y;

                    let perlin_value = perlin_elevation.get([sample_x, sample_y, 0.0]);
                    elevation_value += perlin_value * amplitude;
                }

                let biome_scale = if config.biome_gen.noise_scale <= 0.0 {
                    0.0
                } else {
                    config.biome_gen.noise_scale
                };
                let mut biome_value = 0.;
                let perlin = Perlin::new(config.biome_gen.seed);
                for o in 0..config.biome_gen.octaves {
                    let offset_x: f64 = config.biome_gen.offset.x as f64;
                    let offset_y: f64 = config.biome_gen.offset.y as f64;
                    let frequency: f64 = config.biome_gen.lacunarity.powi(o);
                    let amplitude: f64 = config.biome_gen.persistance.powi(o);
                    let sample_x = x as f64 / biome_scale * frequency + offset_x;
                    let sample_y = y as f64 / biome_scale * frequency + offset_y;

                    let perlin_value = perlin_biome.get([sample_x, sample_y, 0.0]);
                    biome_value += perlin_value * amplitude;
                }

                let tile_index =
                    select_tile_index(&texture_tileset, &config, elevation_value, biome_value);

                m.set(x, y, tile_index as u32);
            }
        }
    }
}

fn select_tile_index(
    texture_tileset: &TextureTileSet,
    config: &Configuration,
    elevation_value: f64,
    biome_value: f64,
) -> usize {
    // Select biome
    let biome_index = if elevation_value <= config.sea_level {
        texture_tileset.biomes_mapping["Ocean"]
    } else {
        /*scale_to_index(
            biome_value,
            -1_f64,
            1_f64,
            0_f64,
            texture_tileset.biomes_position.len() as f64 - 1.,
        )
        .clamp(0, texture_tileset.biomes_position.len() - 1)*/
        texture_tileset.biomes_mapping["savanna"]
    };
    let (min_index, n_tiles) = texture_tileset.biomes_position[biome_index].into();

    // Select biome tile
    scale_to_index(
        elevation_value,
        -1.,
        1.,
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

        build_tiles_texture_from_biomes(&config.biomes).unwrap()
    }
}

fn setup(
    mut commands: Commands,
    config: Res<Configuration>,
    texture_tileset: Res<TextureTileSet>,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
) {
    let camera_transform = Transform {
        scale: Vec3::splat(6.0),
        translation: Vec3::new(-1200., 0., 0.),
        ..default()
    };
    println!("{:?}", camera_transform);
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
        let biome_tiles = biome.tiles.clone().unwrap();
        for (tile_name, &tile_color) in biome_tiles.iter() {
            println!("{}", tile_name);
            for x in 0..tile_size {
                for y in 0..tile_size {
                    img_buffer.put_pixel(x + (tile_size * idx_tile as u32), y, Rgb(tile_color));
                }
            }
            tileset.push(TextureTile {
                index: idx_tile,
                biome_name: biome_name.clone(),
                tile_name: tile_name.clone(),
            });
            idx_tile += 1;
        }
        biomes_position.push([biome_current_index, idx_tile - biome_current_index]);
        biomes_mapping.insert(biome_name.clone(), biomes_position.len() - 1);
        biome_current_index = idx_tile;
    }

    // Create a temporary file
    let mut tmp_file = Builder::new().suffix(".png").keep(true).tempfile()?;
    let mut file = File::create(&tmp_file)?;

    // Bind the writer to the opened file
    let mut writer = BufWriter::new(file);

    // Write bytes into the file as PNG format
    img_buffer.write_to(&mut writer, ImageFormat::Png);

    let path = tmp_file.into_temp_path().to_path_buf();
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
) {
    if config.is_changed() {
        commands.trigger(DrawMapEvent);
    }
}
#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rstest::rstest;

    use crate::{biomes::Biome, build_tiles_texture_from_biomes};

    #[rstest]
    fn build_tiles_texture_from_biomes_succeed() {
        let biomes: HashMap<String, Biome> = [(
            "Forest".to_string(),
            Biome {
                name: "Forest".to_string(),
                tiles: Some(
                    [
                        ("red".to_string(), [255u8, 0u8, 0u8]),
                        ("green".to_string(), [0u8, 255u8, 0u8]),
                        ("blue".to_string(), [0u8, 0u8, 255u8]),
                    ]
                    .into_iter()
                    .collect::<HashMap<String, [u8; 3]>>(),
                ),
            },
        )]
        .into();

        let t = build_tiles_texture_from_biomes(&biomes).unwrap();

        println!("{:?}", t);
    }
}
