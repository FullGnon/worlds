use bevy::prelude::*;
use rand::Rng;

use crate::worlds::utils::scale;

use super::Settings;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<ShapeGeneratorResource>();
}

// This resource should be bound to a state ?
#[derive(Resource)]
pub struct ShapeGeneratorResource {
    pub generator: Box<dyn ShapeGenerator>,
}

impl Default for ShapeGeneratorResource {
    fn default() -> Self {
        Self {
            generator: Box::new(CircleCenteredShape),
        }
    }
}

pub trait ShapeGenerator: Send + Sync {
    fn init(&mut self, config: &Settings);
    fn generate(&self, x: u32, y: u32, config: &Settings) -> f64;
}

#[derive(Default)]
pub struct CircleCenteredShape;

impl ShapeGenerator for CircleCenteredShape {
    fn init(&mut self, config: &Settings) {}
    fn generate(&self, x: u32, y: u32, config: &Settings) -> f64 {
        let center_x = config.width as f64 / 2.;
        let center_y = config.height as f64 / 2.;

        let distance = ((x as f64 - center_x).powi(2) + (y as f64 - center_y).powi(2)).sqrt();
        let distance_max = (center_x.powi(2) + center_y.powi(2)).sqrt();

        scale(distance, 0., distance_max, -1., 1.)
    }
}

pub struct ContinentsShape {
    count: usize,
    random_points: Vec<(f64, f64)>,
    seed: u32,
}

impl Default for ContinentsShape {
    fn default() -> Self {
        Self {
            count: 1,
            random_points: Vec::new(),
            seed: 0,
        }
    }
}

impl ShapeGenerator for ContinentsShape {
    fn init(&mut self, config: &Settings) {
        let x_max = config.width as f64;
        let y_max = config.height as f64;
        let mut rng = rand::thread_rng();

        self.count = 1;
        self.seed = config.elevation_gen.seed;
        self.random_points.clear();

        for i in 0..self.count {
            let x = rng.gen_range(0_f64..=x_max);
            let y = rng.gen_range(0_f64..=y_max);

            self.random_points.push((x, y));
        }

        let distance_max = ((config.width.pow(2) + config.height.pow(2)) as f64).sqrt();
    }

    fn generate(&self, x: u32, y: u32, config: &Settings) -> f64 {
        let distance_max = ((config.width.pow(2) + config.height.pow(2)) as f64).sqrt();
        let mut distance = distance_max;

        for i in 0..self.count {
            let (point_x, point_y) = self.random_points[i];
            let point_distance =
                ((x as f64 - point_x).powi(2) + (y as f64 - point_y).powi(2)).sqrt();
            if point_distance < distance {
                distance = point_distance;
            }
        }

        scale(distance, 0., 2., -1., 1.)
    }
}
