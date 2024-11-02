use elevation::ElevationGenerator;
use shape::ElevationShapeGenerator;
use temperature::TemperatureGenerator;

use crate::worlds::map::MapMode;
use crate::worlds::settings::Settings;

use super::shapes::ShapeGenerator;
use super::tileset::TextureTileSet;

mod elevation;
mod shape;
mod temperature;

pub trait MapGenerator {
    fn generate_tile_index(
        &self,
        x: u32,
        y: u32,
        config: &Settings,
        texture_tileset: &TextureTileSet,
    ) -> u32;
}

struct CompositeGenerator {
    generators: Vec<Box<dyn MapGenerator>>,
}

impl CompositeGenerator {
    fn new(generators: Vec<Box<dyn MapGenerator>>) -> Self {
        Self { generators }
    }
}

pub fn get_map_generator(mode: &MapMode) -> Box<dyn MapGenerator> {
    match mode {
        MapMode::Elevation => Box::new(ElevationGenerator),
        MapMode::Temperature => Box::new(TemperatureGenerator),
        MapMode::WorldShapeMode => Box::new(ElevationShapeGenerator),
    }
}
