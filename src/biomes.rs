use std::{
    collections::HashMap,
    fs::{read_dir, read_to_string},
    path::Path,
};

use bevy::reflect::Reflect;
use serde::{ser::Error, Deserialize};

#[derive(Reflect, Deserialize)]
pub(crate) struct Biome {
    name: String,
    conditions: Option<HashMap<String, HashMap<String, usize>>>,
    fauna: Option<HashMap<String, Vec<String>>>,
    flora: Option<HashMap<String, Vec<String>>>,
    tiles: HashMap<String, [usize; 3]>,
}

pub(crate) fn load_biome(path: &Path) -> Result<Biome> {
    let contents = read_to_string(path)?;
    let biome: Biome = toml::from_str(&contents);

    Ok(biome)
}

pub(crate) fn load_biomes(path: &Path) -> Result<HashMap<String, Biome>> {
    let mut biomes = HashMap::new();

    if !path.is_dir() {
        // How do we use log in rust ?
        println!("{} is not a directory", path);
        return biomes;
    }

    for biome_path in read_dir(path) {}

    biomes
}
