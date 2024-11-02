use bevy::prelude::{FromWorld, Resource, World};

use crate::worlds::settings::Settings;

#[derive(Default, Debug, Clone, Copy)]
pub struct Tile {
    pub elevation: f64,
    pub temperature: f64,
}

#[derive(Resource, Debug)]
pub struct TileMatrixResource {
    pub height: usize,
    pub width: usize,
    pub tiles: Vec<Tile>,
}

impl TileMatrixResource {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tiles = Vec::with_capacity(width * height);
        for _ in 0..height * width {
            tiles.push(Tile::default());
        }

        Self {
            width,
            height,
            tiles,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < self.width && y < self.height {
            self.tiles.get(y * self.width + x)
        } else {
            None
        }
    }

    pub fn set(&mut self, x: usize, y: usize, tile: Tile) -> &Self {
        // TODO: Handle invalid inputs
        self.tiles[y * self.width + x] = tile;

        self
    }
}

impl FromWorld for TileMatrixResource {
    fn from_world(world: &mut World) -> Self {
        let config = world.resource::<Settings>();

        Self::new(config.width as usize, config.height as usize)
    }
}
