use bevy::{
    asset::load_internal_binary_asset,
    prelude::*,
    window::{PresentMode, WindowTheme},
};

mod worlds;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::RecordInput, AppSet::Update).chain(),
        );

        // Add Bevy plugins.
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Worlds POC".into(),
                name: Some("worlds".into()),
                resolution: (1600., 1000.).into(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                present_mode: PresentMode::AutoVsync,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                ..default()
            }),
            ..default()
        }));

        load_internal_binary_asset!(
            app,
            TextStyle::default().font,
            "../assets/fonts/JetBrainsMono-Regular.ttf",
            |bytes: &[u8], _path: String| { Font::try_from_bytes(bytes.to_vec()).unwrap() }
        );

        // Add Worlds plugin.
        app.add_plugins(worlds::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}
