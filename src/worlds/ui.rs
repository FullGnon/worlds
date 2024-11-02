use bevy::{input::mouse::MouseWheel, prelude::*};
use bevy_inspector_egui::{
    bevy_egui::{self, EguiPlugin},
    quick::ResourceInspectorPlugin,
    DefaultInspectorConfigPlugin,
};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((EguiPlugin, DefaultInspectorConfigPlugin))
        .add_systems(
            PreUpdate,
            (absorb_egui_inputs.after(bevy_egui::systems::process_input_system),),
        );
}

fn absorb_egui_inputs(
    mut contexts: bevy_egui::EguiContexts,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut mouse_wheel: ResMut<Events<MouseWheel>>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
) {
    let ctx = contexts.ctx_mut();
    if !(ctx.wants_pointer_input() || ctx.is_pointer_over_area()) {
        return;
    }
    let modifiers = [
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::AltLeft,
        KeyCode::AltRight,
        KeyCode::ShiftLeft,
        KeyCode::ShiftRight,
    ];

    let pressed = modifiers.map(|key| keyboard.pressed(key).then_some(key));

    mouse.reset_all();
    mouse_wheel.clear();
    keyboard.reset_all();

    for key in pressed.into_iter().flatten() {
        keyboard.press(key);
    }
}
