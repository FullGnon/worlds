use crate::worlds::{
    map::{tile::Tile, MapGenerator},
    settings::Settings,
};

pub struct ElevationShapeGenerator;

impl MapGenerator for ElevationShapeGenerator {
    fn apply(&self, tile: &mut Tile, x: u32, y: u32, settings: &Settings) {
        let value = 0.;

        tile.elevation += value;
    }
}
