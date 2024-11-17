use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PanCamPlugin)
        .add_systems(Startup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
    /* TODO: hardcoded camera */
    let camera_transform = Transform { ..default() };

    commands.spawn((
        Camera2dBundle {
            transform: camera_transform,
            ..default()
        },
        PanCam { ..default() },
    ));
}
