use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    aabb::Aabb,
    block::Block,
    game_state::GameState,
    physics::{is_position_solid, Velocity},
    world::regenerate_chunk_mesh,
    VoxelMaterials, WorldMap,
};

#[derive(Debug, Clone, Copy)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_player, initial_grab_cursor),
        )
        .add_systems(
            Update,
            (
                (player_look, toggle_grab, player_move),
                handle_block_interaction,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Player {
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, Component)]
struct PlayerCamera;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player::default(),
            Transform::from_xyz(10.0, 100.0, 10.0),
            Velocity(Vec3::ZERO),
            Visibility::Inherited,
        ))
        .with_child((
            PlayerCamera,
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.6, 0.0),
            Visibility::Inherited,
        ));
}

fn toggle_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }

    let mut window = primary_window.single_mut();

    if window.cursor_options.grab_mode == CursorGrabMode::None {
        grab(&mut window);
    } else {
        ungrab(&mut window);
    }
}

fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        grab(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

fn grab(window: &mut Window) {
    window.cursor_options.grab_mode = CursorGrabMode::Confined;
    window.cursor_options.visible = false;
}

fn ungrab(window: &mut Window) {
    window.cursor_options.grab_mode = CursorGrabMode::None;
    window.cursor_options.visible = true;
}

fn player_look(
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

fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut query: Query<(&mut Velocity, &Player)>,
    camera: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
) {
    if let Ok(window) = primary_window.get_single() {
        let (mut velocity, physics) = query.single_mut();
        let transform = camera.single();
        let mut movement = Vec3::ZERO;

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

            // Normalize horizontal movement
            movement = movement.normalize_or_zero();

            const MOVEMENT_SPEED: f32 = 8.0;
            const JUMP_FORCE: f32 = 9.0;

            // Apply movement
            let target_velocity = movement * MOVEMENT_SPEED;

            // Jump when space is pressed and on ground
            if keys.pressed(KeyCode::Space) && physics.on_ground {
                velocity.0.y = JUMP_FORCE;
            }

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
}

const MAX_REACH: f32 = 5.0;

fn handle_block_interaction(
    mouse: Res<ButtonInput<MouseButton>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &Parent), With<PlayerCamera>>,
    player_query: Query<&Transform, With<Player>>,
    mut world: ResMut<WorldMap>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<VoxelMaterials>,
) {
    // Only handle input when mouse is clicked
    if !mouse.just_pressed(MouseButton::Left) && !mouse.just_pressed(MouseButton::Right) {
        return;
    }

    let window = primary_window.single();
    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    let (camera_transform, camera_parent) = camera_query.single();
    let player_transform = player_query.get(camera_parent.get()).unwrap();

    let ray_origin = player_transform.translation + camera_transform.translation;
    let ray_direction = camera_transform.forward().normalize();

    // Check for block intersections
    if let Some((hit_pos, hit_normal)) =
        raycast_blocks(&world, ray_origin, ray_direction, MAX_REACH)
    {
        let place_pos = if mouse.just_pressed(MouseButton::Right) {
            hit_pos + hit_normal
        } else {
            hit_pos
        };

        // Convert world position to chunk and local coordinates
        let chunk_pos = IVec3::new(
            (place_pos.x / 16.0).floor() as i32,
            (place_pos.y / 16.0).floor() as i32,
            (place_pos.z / 16.0).floor() as i32,
        );

        let local_pos = UVec3::new(
            place_pos.x.rem_euclid(16.0) as u32,
            place_pos.y.rem_euclid(16.0) as u32,
            place_pos.z.rem_euclid(16.0) as u32,
        );

        // Modify the block
        if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
            if mouse.just_pressed(MouseButton::Left) {
                chunk.set(local_pos, Block::Air);
            } else {
                chunk.set(local_pos, Block::Rock);
            }

            // Regenerate mesh for the modified chunk
            regenerate_chunk_mesh(
                &mut commands,
                &mut world.chunks,
                chunk_pos,
                &mut meshes,
                &materials,
            );

            // Regenerate neighboring chunks if the modified block was on the edge
            let neighbors = [
                IVec3::X,
                IVec3::NEG_X,
                IVec3::Y,
                IVec3::NEG_Y,
                IVec3::Z,
                IVec3::NEG_Z,
            ];

            for &offset in &neighbors {
                if local_pos.x == 0
                    || local_pos.x == 15
                    || local_pos.y == 0
                    || local_pos.y == 15
                    || local_pos.z == 0
                    || local_pos.z == 15
                {
                    let neighbor_pos = chunk_pos + offset;
                    if world.chunks.contains_key(&neighbor_pos) {
                        regenerate_chunk_mesh(
                            &mut commands,
                            &mut world.chunks,
                            neighbor_pos,
                            &mut meshes,
                            &materials,
                        );
                    }
                }
            }
        }
    }
}

fn raycast_blocks(
    world: &WorldMap,
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
) -> Option<(Vec3, Vec3)> {
    let mut current_pos = ray_origin;
    let step = 0.1; // Small step size for reasonable accuracy

    for _ in 0..((max_distance / step) as i32) {
        let block_pos = current_pos.floor();

        if is_position_solid(world, block_pos) {
            let block_center = block_pos + Vec3::splat(0.5);
            let block_aabb = Aabb::new(block_center, Vec3::ONE);

            if block_aabb
                .ray_intersection(ray_origin, ray_direction)
                .is_some()
            {
                // Calculate which face was hit by checking the entry point
                let hit_point = current_pos - ray_direction * step;
                let relative_pos = hit_point - block_center;

                // Find the axis with the largest magnitude - that's our hit normal
                let normal = if relative_pos.x.abs() > relative_pos.y.abs()
                    && relative_pos.x.abs() > relative_pos.z.abs()
                {
                    Vec3::X * relative_pos.x.signum()
                } else if relative_pos.y.abs() > relative_pos.x.abs()
                    && relative_pos.y.abs() > relative_pos.z.abs()
                {
                    Vec3::Y * relative_pos.y.signum()
                } else {
                    Vec3::Z * relative_pos.z.signum()
                };

                return Some((block_pos, normal));
            }
        }

        current_pos += ray_direction * step;
    }

    None
}
