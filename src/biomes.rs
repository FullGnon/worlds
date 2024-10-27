use std::{
    collections::{hash_map::Entry, HashMap},
    fmt,
    fs::{read_dir, read_to_string},
    io,
    path::{Path, PathBuf},
};

use bevy::reflect::Reflect;
use serde::Deserialize;

#[derive(Reflect, Deserialize, Debug, PartialEq, Clone)]
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

    let mut biomes: HashMap<String, Biome> = HashMap::new();

    for entry in read_dir(path)? {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                match load_biome(&path) {
                    Ok(biome) => {
                        let biome_name = biome.name.clone();

                        match biomes.entry(biome.name.clone()) {
                            Entry::Occupied(o) => {
                                eprintln!("Warning: Duplicate Biome found '{}'", biome_name);
                            }
                            Entry::Vacant(v) => {
                                v.insert(biome);
                                println!("Biome loaded successfully: '{}'", biome_name);
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("Error loading biome: {:?}", path);
                        #[cfg(debug_assertions)]
                        eprintln!("{}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error read dir {}", e);
            }
        }
    }

    Ok(biomes)
}

#[cfg(test)]
mod tests {
    use bevy::{prelude::Res, utils::HashMap};
    use noise::Billow;
    use rstest::{fixture, rstest};
    use std::{
        borrow::Borrow,
        fmt::Display,
        fs::{remove_dir_all, remove_file, File},
        io::{Error, Write},
        path::{self, Path, PathBuf},
        time::Duration,
    };
    use tempfile::{
        env::{self, temp_dir},
        tempdir, tempfile, Builder, TempDir,
    };

    use crate::biomes::load_biome;

    use super::{load_biomes, Biome, LoadBiomeError};

    #[derive(Clone)]
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

    impl Display for BiomeTestCase {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let name = match self {
                BiomeTestCase::NameOnly => "NameOnly",
                BiomeTestCase::WithEmptyTiles => "WithEmptyTiles",
                BiomeTestCase::WithSomeTiles => "WithSomeTiles",
                BiomeTestCase::MissingName => "MissingName",
                BiomeTestCase::InvalidFormat => "InvalidFormat",
                BiomeTestCase::WithTilesError => "WithTilesError",
            };

            write!(f, "{}", name)
        }
    }

    impl BiomeTestCase {
        fn file_content(&self) -> &str {
            match self {
                // Valid
                BiomeTestCase::NameOnly => {
                    r#"
                    name = "NameOnly"
                "#
                }
                BiomeTestCase::WithEmptyTiles => {
                    r#"
                    name = "WithEmptyTiles"

                    [tiles]
                    "#
                }
                BiomeTestCase::WithSomeTiles => {
                    r#"
                    name = "WithSomeTiles"

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
                    name = "WithTilesError"

                    [tiles]
                    grass = 0
                    "#
                }
            }
        }

        fn expectation(&self) -> Option<Biome> {
            match self {
                BiomeTestCase::NameOnly => Some(Biome {
                    name: "NameOnly".to_string(),
                    tiles: None,
                }),
                BiomeTestCase::WithSomeTiles => Some(Biome {
                    name: "WithSomeTiles".to_string(),
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
                    name: "WithEmptyTiles".to_string(),
                    tiles: Some([].into_iter().collect()),
                }),
                // Invalid
                BiomeTestCase::MissingName => todo!(),
                BiomeTestCase::InvalidFormat => todo!(),
                BiomeTestCase::WithTilesError => todo!(),
            }
        }
    }

    fn materialize_biome_test_case(
        biome_test_case: &BiomeTestCase,
        root_path: Option<&Path>,
    ) -> Result<PathBuf, Error> {
        let default_tmp_dir = tempfile::env::temp_dir();
        let root_path = root_path.unwrap_or(default_tmp_dir.as_path());
        let tmp_file = Builder::new()
            .suffix(".toml")
            .keep(true)
            .tempfile_in(root_path)?;

        let mut file = File::create(&tmp_file)?;
        writeln!(file, "{}", biome_test_case.file_content());

        Ok(tmp_file.into_temp_path().to_path_buf())
    }

    fn materialize_biome_test_cases(
        biome_test_cases: &Vec<BiomeTestCase>,
    ) -> Result<PathBuf, Error> {
        let tmp_dir = Builder::new().keep(true).tempdir()?;
        let tmp_path = tmp_dir.into_path();

        for biome_test_case in biome_test_cases {
            materialize_biome_test_case(biome_test_case, Some(tmp_path.as_path()));
        }

        Ok(tmp_path)
    }

    #[rstest]
    #[case::name_only(BiomeTestCase::NameOnly)]
    #[case::with_empty_tiles(BiomeTestCase::WithEmptyTiles)]
    #[case::with_some_tiles(BiomeTestCase::WithSomeTiles)]
    fn test_load_biome_success(#[case] biome_test_case: BiomeTestCase) -> Result<(), Error> {
        let filepath = materialize_biome_test_case(&biome_test_case, None)?;
        let biome = load_biome(&filepath).expect("Valid biome file should load successfully");

        assert_eq!(biome, biome_test_case.expectation().unwrap());

        remove_file(filepath)?;
        Ok(())
    }

    #[rstest]
    #[case::missing_name(BiomeTestCase::MissingName)]
    #[case::invalid_format(BiomeTestCase::InvalidFormat)]
    #[case::with_tiles_error(BiomeTestCase::WithTilesError)]
    fn test_load_biome_invalid(#[case] biome_test_case: BiomeTestCase) -> Result<(), Error> {
        let filepath = materialize_biome_test_case(&biome_test_case, None)?;
        let biome = load_biome(&filepath);

        assert!(biome.is_err());

        remove_file(filepath)?;
        Ok(())
    }

    #[rstest]
    fn test_load_biomes_success() -> Result<(), Error> {
        let mut biome_test_cases = [BiomeTestCase::NameOnly, BiomeTestCase::WithSomeTiles].to_vec();
        let path = materialize_biome_test_cases(&biome_test_cases)?;
        let biomes = load_biomes(&path).expect("Valid biome files should load sucessfully");

        assert_eq!(biomes.len(), biome_test_cases.len());
        for biome_test_case in &biome_test_cases {
            let biome = biome_test_case.expectation().unwrap();
            let biome_name = biome.name.to_string();
            assert!(biomes.contains_key(&biome_name));
            println!("{}", &biome_name);
            assert_eq!(biomes[&biome_name], biome);
        }

        remove_dir_all(path)?;
        Ok(())
    }

    #[rstest]
    fn test_load_biomes_skip_dupicates_by_name() -> Result<(), Error> {
        let mut biome_test_cases = [BiomeTestCase::NameOnly, BiomeTestCase::NameOnly].to_vec();

        let path = materialize_biome_test_cases(&biome_test_cases)?;
        let biomes = load_biomes(&path).expect("Skip duplicated biomes");

        assert_eq!(biomes.len(), 1);
        Ok(())
    }

    #[rstest]
    fn test_load_biomes_invalid() -> Result<(), Error> {
        let mut biome_test_cases = [BiomeTestCase::NameOnly, BiomeTestCase::InvalidFormat].to_vec();

        let path = materialize_biome_test_cases(&biome_test_cases)?;
        let biomes = load_biomes(&path).expect("Skip invalid biomes");

        assert_eq!(biomes.len(), 1);
        Ok(())
    }
}
