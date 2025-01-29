use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::PlayerCamera;

pub fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        warn!("Primary window not found, cursor grab will not be enabled");
        return;
    };
    set_grab(&mut window, true);
}

pub fn toggle_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let mut window = primary_window.single_mut();

    let should_grab = window.cursor_options.grab_mode == CursorGrabMode::None;
    set_grab(&mut window, should_grab);
}

pub fn player_look(
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: EventReader<MouseMotion>,
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
) {
    let window = primary_window.single();

    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    };

    let mut transform = camera.single_mut();

    for ev in state.read() {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        let window_scale = window.height().min(window.width());
        pitch -= (0.00012 * ev.delta.y * window_scale).to_radians();
        yaw -= (0.00012 * ev.delta.x * window_scale).to_radians();

        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
        let pitch_rotation = Quat::from_axis_angle(Vec3::X, pitch.clamp(-1.54, 1.54));
        transform.rotation = yaw_rotation * pitch_rotation;
    }
}

fn set_grab(window: &mut Window, grab: bool) {
    window.cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    window.cursor_options.visible = !grab;
}
