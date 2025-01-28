mod block;
mod chunk;
mod voxel_material;
mod voxel_mesh;

use avian3d::prelude::*;
use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    utils::HashMap,
    window::{CursorGrabMode, PrimaryWindow},
};
use block::Block;
use chunk::Chunk;
use noise::{NoiseFn, Perlin};
use voxel_material::VoxelMaterial;

#[derive(Resource)]
struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
            PhysicsPlugins::default(),
        ))
        .insert_resource(ChunkManager {
            chunks: HashMap::new(),
        })
        .add_systems(
            Startup,
            (
                setup_player,
                initial_grab_cursor,
                generate_chunks,
                build_chunk_meshes.after(generate_chunks),
            ),
        )
        .add_systems(Update, (player_look, toggle_grab, player_move))
        .run();
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            Transform::from_xyz(10.0, 100.0, 10.0),
            // The player character needs to be configured as a dynamic rigid body of the physics
            // engine.
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.0),
            // Tnua can fix the rotation, but the character will still get rotated before it can do so.
            // By locking the rotation we can prevent this.
            LockedAxes::ROTATION_LOCKED,
            Visibility::Inherited,
        ))
        .with_child((PlayerCamera, Camera3d::default(), Visibility::Inherited));
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
    mut query: Query<&mut LinearVelocity, With<Player>>,
    camera: Query<&Transform, With<PlayerCamera>>,
) {
    if let Ok(window) = primary_window.get_single() {
        let mut linear_velocity = query.single_mut();
        let transform = camera.single();
        let mut velocity = Vec3::ZERO;

        // Get the camera's forward and right vectors
        let forward = transform.forward();
        // Project forward vector onto XZ plane and normalize
        let forward = Vec3::new(forward.x, 0.0, forward.z).normalize_or_zero();
        // Get right vector by rotating forward 90 degrees around Y axis
        let right = Vec3::new(-forward.z, 0.0, forward.x);

        // Handle movement input
        if window.cursor_options.grab_mode != CursorGrabMode::None {
            if keys.pressed(KeyCode::KeyW) {
                velocity += forward;
            }
            if keys.pressed(KeyCode::KeyS) {
                velocity -= forward;
            }
            if keys.pressed(KeyCode::KeyA) {
                velocity -= right;
            }
            if keys.pressed(KeyCode::KeyD) {
                velocity += right;
            }

            // Normalize horizontal movement
            velocity = velocity.normalize_or_zero();

            // Apply movement speed
            const MOVEMENT_SPEED: f32 = 5.0;
            velocity *= MOVEMENT_SPEED;

            // Preserve vertical velocity (for gravity/jumping)
            velocity.y = linear_velocity.0.y;

            // Handle vertical movement (temporary flying controls)
            if keys.pressed(KeyCode::Space) {
                velocity.y = MOVEMENT_SPEED;
            }
            if keys.pressed(KeyCode::ShiftLeft) {
                velocity.y = -MOVEMENT_SPEED;
            }

            // Smoothly interpolate to target velocity
            const ACCELERATION: f32 = 20.0;
            linear_velocity.0 = linear_velocity
                .0
                .lerp(velocity, ACCELERATION * time.delta_secs());
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

fn generate_chunks(mut chunk_manager: ResMut<ChunkManager>) {
    let perlin = Perlin::new(42);

    // Generate a 4x4x4 grid of chunks
    for chunk_x in 0..4 {
        for chunk_y in 0..4 {
            for chunk_z in 0..4 {
                let mut chunk = Chunk::new();
                let chunk_pos = IVec3::new(chunk_x, chunk_y, chunk_z);

                // Fill each 16x16x16 chunk
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let world_x = chunk_x * 16 + x as i32;
                            let world_y = chunk_y * 16 + y as i32;
                            let world_z = chunk_z * 16 + z as i32;

                            let noise_value = perlin.get([
                                world_x as f64 * 0.1,
                                world_y as f64 * 0.1,
                                world_z as f64 * 0.1,
                            ]);

                            let normalized_noise = (noise_value + 1.0) / 2.0;
                            let height_threshold = (world_y as f64 / (4.0 * 16.0)) * 0.8;

                            if normalized_noise > height_threshold {
                                chunk.set(UVec3::new(x, y, z), Block::Rock);
                            }
                        }
                    }
                }

                chunk_manager.chunks.insert(chunk_pos, chunk);
            }
        }
    }
}

fn build_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    chunk_manager: Res<ChunkManager>,
) {
    let material = materials.add(VoxelMaterial {});

    for (&chunk_pos, chunk) in chunk_manager.chunks.iter() {
        let (mesh, collider) = chunk.render(&chunk_manager.chunks, chunk_pos).build();

        let mut entity = commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(
                chunk_pos.x as f32 * 16.0,
                chunk_pos.y as f32 * 16.0,
                chunk_pos.z as f32 * 16.0,
            ),
        ));

        if let Some(collider) = collider {
            entity.insert((collider, RigidBody::Static, CollisionMargin(0.5)));
        }
    }
}
