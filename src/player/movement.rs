use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::physics::Velocity;

use super::{Player, PlayerCamera};

pub fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Velocity, &mut Transform, &Player)>,
    camera: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    const MOVEMENT_SPEED: f32 = 11.0;
    const JUMP_FORCE: f32 = 7.6;

    if let Ok(window) = primary_window.get_single() {
        let (mut velocity, mut p_transform, physics) = query.single_mut();
        let transform = camera.single();
        let mut movement = Vec3::ZERO;

        if keys.just_pressed(KeyCode::KeyJ) {
            p_transform.translation.x += 10000.0;
        }

        // Get the camera's forward and right vectors
        let forward = transform.forward();
        let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        let right = Vec3::new(-forward.z, 0.0, forward.x);

        if window.cursor_options.grab_mode != CursorGrabMode::None {
            if keys.pressed(KeyCode::KeyW) {
                movement += forward;
            }
            if keys.pressed(KeyCode::KeyS) {
                movement -= forward;
            }
            if keys.pressed(KeyCode::KeyA) {
                movement -= right;
            }
            if keys.pressed(KeyCode::KeyD) {
                movement += right;
            }

            // Jump when space is pressed and on ground
            if keys.pressed(KeyCode::Space) && physics.on_ground {
                velocity.0.y = JUMP_FORCE;
            }
        }

        // Normalize horizontal movement
        movement = movement.normalize_or_zero();

        // Apply movement
        let target_velocity = movement * MOVEMENT_SPEED;

        // Smoothly interpolate horizontal velocity
        let acceleration = if physics.on_ground { 10.0 } else { 2.0 };
        velocity.0.x = velocity
            .0
            .x
            .lerp(target_velocity.x, acceleration * time.delta_secs());
        velocity.0.z = velocity
            .0
            .z
            .lerp(target_velocity.z, acceleration * time.delta_secs());
    }
}
