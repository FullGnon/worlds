use std::{
    collections::HashMap,
    fmt,
    fs::{read_dir, read_to_string},
    io,
    path::Path,
};

use bevy::reflect::Reflect;
use serde::Deserialize;

#[derive(Reflect, Deserialize, Debug)]
pub(crate) struct Biome {
    name: String,
    conditions: Option<HashMap<String, HashMap<String, usize>>>,
    fauna: Option<HashMap<String, Vec<String>>>,
    flora: Option<HashMap<String, Vec<String>>>,
    tiles: Option<HashMap<String, [usize; 3]>>,
}

#[derive(Debug)]
pub enum LoadBiomeError {
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl fmt::Display for LoadBiomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadBiomeError::Io(e) => write!(f, "File read error: {}", e),
            LoadBiomeError::Toml(e) => write!(f, "TOML parse error: {}", e),
        }
    }
}

pub(crate) fn load_biome(path: &Path) -> Result<Biome, LoadBiomeError> {
    let contents = read_to_string(path).map_err(LoadBiomeError::Io)?;
    let biome: Biome = toml::from_str(&contents).map_err(LoadBiomeError::Toml)?;
    Ok(biome)
}

pub(crate) fn load_biomes(path: &Path) -> Result<HashMap<String, Biome>, io::Error> {
    if !path.is_dir() {
        // How do we use log in rust ?
        println!("{:?} is not a directory", path);
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Path to biomes must be a directory.",
        ));
    }

    let biomes = read_dir(path)?
        .filter_map(|entry| match entry {
            Ok(entry) => {
                let path = entry.path();
                match load_biome(&path) {
                    Ok(biome) => {
                        println!("Biome loaded successfully: {:?}", biome);

                        Some((biome.name.clone(), biome))
                    }
                    Err(e) => {
                        eprintln!("Error loading biome: {}", e);

                        None
                    }
                }
            }
            Err(e) => None,
        })
        .collect();

    Ok(biomes)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{Error, Write},
    };
    use tempfile::{env::temp_dir, tempdir, tempfile};

    use crate::biomes::load_biome;

    #[test]
    fn test_load_biome_success() -> Result<(), Error> {
        let dir = tempdir()?;
        let biome_path = dir.path().join("forest.toml");
        let mut file = File::create(&biome_path)?;

        let valid_biome_content = r#"
            name = "Forest"
        "#;

        writeln!(file, "{}", valid_biome_content);

        let biome = load_biome(&biome_path).expect("Valid biome file should load successfully");

        assert_eq!(biome.name, "Forest");

        Ok(())
    }
}
