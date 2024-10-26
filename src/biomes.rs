use std::{
    collections::HashMap,
    fmt,
    fs::{read_dir, read_to_string},
    io,
    path::{Path, PathBuf},
};

use bevy::reflect::Reflect;
use serde::Deserialize;

#[derive(Reflect, Deserialize, Debug, PartialEq)]
pub(crate) struct Biome {
    name: String,
    //conditions: Option<HashMap<String, HashMap<String, usize>>>,
    //fauna: Option<HashMap<String, Vec<String>>>,
    //flora: Option<HashMap<String, Vec<String>>>,
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

pub(crate) fn load_biome(path: &PathBuf) -> Result<Biome, LoadBiomeError> {
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
    use bevy::utils::HashMap;
    use rstest::{fixture, rstest};
    use std::{
        fs::File,
        io::{Error, Write},
        path::PathBuf,
        time::Duration,
    };
    use tempfile::{env::temp_dir, tempdir, tempfile};

    use crate::biomes::load_biome;

    use super::{Biome, LoadBiomeError};

    enum BiomeTestCase {
        // Valid test cases
        NameOnly,
        WithEmptyTiles,
        WithSomeTiles,
        // Invalid test cases
        MissingName,
        InvalidFormat,
        WithTilesError,
    }

    impl BiomeTestCase {
        fn file_content(&self) -> &str {
            match self {
                // Valid
                BiomeTestCase::NameOnly => {
                    r#"
                    name = "Forest"
                "#
                }
                BiomeTestCase::WithEmptyTiles => {
                    r#"
                    name = "Forest"

                    [tiles]
                    "#
                }
                BiomeTestCase::WithSomeTiles => {
                    r#"
                    name = "Forest"

                    [tiles]
                    grass = [1, 1, 1]
                    water = [2, 2, 2]
                    "#
                }
                // Invalid
                BiomeTestCase::InvalidFormat => r#"{"foo", "bar"}"#,
                BiomeTestCase::MissingName => {
                    r#"
                    foo = "bar"
                    "#
                }
                BiomeTestCase::WithTilesError => {
                    r#"
                    name = "Forest"

                    [tiles]
                    grass = 0
                    "#
                }
            }
        }

        fn expectation(&self) -> Option<Biome> {
            match self {
                BiomeTestCase::NameOnly => Some(Biome {
                    name: "Forest".to_string(),
                    tiles: None,
                }),
                BiomeTestCase::WithSomeTiles => Some(Biome {
                    name: "Forest".to_string(),
                    tiles: Some(
                        [
                            ("grass".to_string(), [1, 1, 1]),
                            ("water".to_string(), [2, 2, 2]),
                        ]
                        .into_iter()
                        .collect(),
                    ),
                }),
                BiomeTestCase::WithEmptyTiles => Some(Biome {
                    name: "Forest".to_string(),
                    tiles: Some([].into_iter().collect()),
                }),
                // Invalid
                BiomeTestCase::MissingName => todo!(),
                BiomeTestCase::InvalidFormat => todo!(),
                BiomeTestCase::WithTilesError => todo!(),
            }
        }
    }

    #[rstest]
    #[case::name_only(BiomeTestCase::NameOnly)]
    #[case::with_empty_tiles(BiomeTestCase::WithEmptyTiles)]
    #[case::with_some_tiles(BiomeTestCase::WithSomeTiles)]
    fn test_load_biome_success(#[case] biome_test_case: BiomeTestCase) -> Result<(), Error> {
        // TODO: Write the file creation as a fixture ?
        // tempdir must live during test execution otherwise temp directory is cleaned
        let dir = tempdir()?;
        let path = dir.path().join("biome.toml");
        let mut file = File::create(&path)?;
        writeln!(file, "{}", biome_test_case.file_content());

        let biome = load_biome(&path).expect("Valid biome file should load successfully");

        assert_eq!(biome, biome_test_case.expectation().unwrap());

        Ok(())
    }

    #[rstest]
    #[case::missing_name(BiomeTestCase::MissingName)]
    #[case::invalid_format(BiomeTestCase::InvalidFormat)]
    #[case::with_tiles_error(BiomeTestCase::WithTilesError)]
    fn test_load_biome_invalid(#[case] biome_test_case: BiomeTestCase) -> Result<(), Error> {
        // TODO: Write the file creation as a fixture ?
        // tempdir must live during test execution otherwise temp directory is cleaned
        let dir = tempdir()?;
        let path = dir.path().join("biome.toml");
        let mut file = File::create(&path)?;
        writeln!(file, "{}", biome_test_case.file_content());

        let biome = load_biome(&path);
        // TODO: check error type is the one expected by biome_test_case
        assert!(biome.is_err());

        Ok(())
    }

    #[rstest]
    fn test_load_biomes_success() {
        todo!()
    }

    #[rstest]
    fn test_load_biomes_invalid() {
        todo!()
    }
}
