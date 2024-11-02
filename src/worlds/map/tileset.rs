use bevy::prelude::*;
use image::{GenericImage, ImageEncoder, ImageFormat, Pixel, PixelWithColorType, Rgb, RgbImage};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::worlds::map::biomes::Biome;
use crate::worlds::settings::Settings;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<TextureTileSet>();
}

#[derive(Debug)]
pub struct TextureTile {
    index: usize,
    biome_name: String,
    tile_name: String,
}

#[derive(Resource, Debug)]
pub struct TextureTileSet {
    pub path: PathBuf,
    pub tileset: Vec<TextureTile>,
    pub biomes_position: Vec<[usize; 2]>,
    pub biomes_mapping: HashMap<String, usize>,
}

impl FromWorld for TextureTileSet {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Settings>();

        let enabled_biomes = config
            .biomes
            .clone()
            .into_iter()
            .filter(|(_, biome)| biome.enabled.unwrap_or_default())
            .collect::<HashMap<String, Biome>>();

        build_tiles_texture_from_biomes(&enabled_biomes).unwrap()
    }
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
