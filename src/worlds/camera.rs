use bevy::prelude::*;
use bevy_pancam::{PanCam, PanCamPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PanCamPlugin)
        .add_systems(Startup, spawn_camera);
}

fn spawn_camera(mut commands: Commands) {
    /* TODO: hardcoded camera */
    let camera_transform = Transform {
        scale: Vec3::splat(6.0),
        translation: Vec3::new(-1200., 0., 0.),
        ..default()
    };

    commands.spawn((
        Camera2dBundle {
            transform: camera_transform,
            ..default()
        },
        PanCam { ..default() },
    ));
}
