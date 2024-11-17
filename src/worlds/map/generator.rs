use bevy::prelude::*;
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};

use crate::worlds::settings::Settings;

pub mod elevation;
pub mod temperature;

pub trait MapGenerator: Send + Sync {
    fn get_value(&self, tile_pos: &TilePos, settings: &Settings) -> f64;
    fn get_min_max(settings: &Settings) -> [f64; 2];
}
