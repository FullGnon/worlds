use bevy::color::Color;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::worlds::map::generator::elevation::TileElevation;
use crate::worlds::utils::{scale, scale_to_index};

pub struct ElevationMapRenderer;

impl ElevationMapRenderer {
    pub fn get_color(&self, tile_elevation: &TileElevation) -> Color {
        let &TileElevation(elevation) = tile_elevation;

        let normalized = scale(elevation, -20., 20., 0., 1.);

        // Define color stops as (normalized_value, (r, g, b)) with normalized RGB values (0.0 - 1.0)
        let color_stops = [
            (0., (89, 127, 198)),
            (0., (83, 158, 216)),
            (0., (79, 171, 226)),
            (0., (39, 194, 245)),
            (0., (79, 205, 248)),
            (0., (112, 208, 245)),
            (0., (141, 216, 248)),
            (0., (158, 217, 204)),
            (0., (196, 216, 190)),
            (0., (224, 219, 177)),
            (0., (254, 227, 168)),
            (0., (252, 194, 128)),
            (0., (229, 159, 89)),
            (0., (210, 133, 55)),
            (0., (195, 106, 26)),
            (0., (202, 102, 27)),
            (0., (211, 88, 31)),
        ];

        for i in 1..color_stops.len() {
            if normalized <= i as f64 / color_stops.len() as f64 {
                let (r, g, b) = color_stops[i].1;

                let r_ = scale(r as f64, 0., 255., 0., 1.);
                let g_ = scale(g as f64, 0., 255., 0., 1.);
                let b_ = scale(b as f64, 0., 255., 0., 1.);

                return Color::srgb(r_ as f32, g_ as f32, b_ as f32);
            }
        }

        Color::srgb(1., 1., 1.)
    }
}
