use bevy::app::App;

pub mod camera;
pub mod map;
pub mod settings;
pub mod ui;
pub mod utils;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((ui::plugin, camera::plugin, settings::plugin, map::plugin));
}
