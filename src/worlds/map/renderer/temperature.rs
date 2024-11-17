use bevy::color::Color;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::worlds::map::generator::temperature::{TemperatureGenerator, TileTemperature};
use crate::worlds::map::generator::MapGenerator;
use crate::worlds::settings::Settings;
use crate::worlds::utils::{scale, scale_to_index};

pub struct TemperatureMapRenderer;

impl TemperatureMapRenderer {
    pub fn get_color(&self, tile_temperature: &TileTemperature, settings: &Settings) -> Color {
        let &TileTemperature(temperature) = tile_temperature;
        let [min, max] = TemperatureGenerator::get_min_max(settings);

        let normalized = scale(temperature, min, max, 0., 1.);

        // Define color stops as (normalized_value, (r, g, b)) with normalized RGB values (0.0 - 1.0)
        let color_stops = [
            (0., (124, 64, 255)),
            (0., (59, 57, 230)),
            (0., (63, 65, 252)),
            (0., (65, 145, 247)),
            (0., (64, 197, 252)),
            (0., (177, 255, 64)),
            (0., (254, 254, 65)),
            (0., (254, 211, 66)),
            (0., (252, 166, 63)),
            (0., (255, 115, 64)),
            (0., (255, 70, 64)),
            (0., (162, 41, 40)),
            (0., (121, 29, 30)),
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
