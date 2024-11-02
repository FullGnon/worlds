use super::settings::Settings;

pub fn scale(value: f64, min: f64, max: f64, scale_min: f64, scale_max: f64) -> f64 {
    ((value - min) / (max - min)) * (scale_max - scale_min) + scale_min
}

pub fn scale_to_index(value: f64, min: f64, max: f64, scale_min: f64, scale_max: f64) -> usize {
    scale(value, min, max, scale_min, scale_max).round() as usize
}

pub fn xy_to_lonlat(config: &Settings, x: u32, y: u32) -> (f64, f64) {
    let x_min = 0_f64;
    let y_min = 0_f64;
    let x_max = (config.width - 1) as f64;
    let y_max = (config.height - 1) as f64;

    let lon = scale(x as f64, x_min, x_max, -180., 180.);
    let lat = scale(y as f64, y_min, y_max, -90., 90.);

    (lon, lat)
}

pub fn lonlat_to_xy(config: &Settings, lon: f64, lat: f64) -> (u32, u32) {
    let x_min = 0_f64;
    let y_min = 0_f64;
    let x_max = (config.width - 1) as f64;
    let y_max = (config.height - 1) as f64;
    let x = scale(lon, -180., 180., x_min, x_max);
    let y = scale(lat, -90., 90., y_min, y_max);

    (x as u32, y as u32)
}
