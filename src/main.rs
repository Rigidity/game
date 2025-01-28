mod block;
mod chunk;
mod voxel_material;
mod voxel_mesh;

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

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct VoxelPhysics {
    on_ground: bool,
}

#[derive(Debug)]
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    fn new(center: Vec3, size: Vec3) -> Self {
        let half_size = size * 0.5;
        Self {
            min: center - half_size,
            max: center + half_size,
        }
    }

    fn intersects(&self, other: &Aabb) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
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
        .add_systems(
            Update,
            (
                player_look,
                toggle_grab,
                player_move,
                voxel_physics.after(player_move),
            ),
        )
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
            Velocity(Vec3::ZERO),
            VoxelPhysics { on_ground: false },
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
    mut query: Query<(&mut Velocity, &VoxelPhysics), With<Player>>,
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

            const MOVEMENT_SPEED: f32 = 6.0;
            const JUMP_FORCE: f32 = 8.0;

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
        let mesh = chunk.render(&chunk_manager.chunks, chunk_pos).build();

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(
                chunk_pos.x as f32 * 16.0,
                chunk_pos.y as f32 * 16.0,
                chunk_pos.z as f32 * 16.0,
            ),
        ));
    }
}

fn voxel_physics(
    time: Res<Time>,
    chunk_manager: Res<ChunkManager>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut VoxelPhysics), With<Player>>,
) {
    let (mut transform, mut velocity, mut physics) = query.single_mut();

    const GRAVITY: f32 = -20.0;
    const TERMINAL_VELOCITY: f32 = -30.0;
    const PLAYER_SIZE: Vec3 = Vec3::new(0.8, 1.8, 0.8);

    // Calculate new position
    let mut new_pos = transform.translation + velocity.0 * time.delta_secs();

    // Create player Aabb
    let player_aabb = Aabb::new(new_pos, PLAYER_SIZE);

    // Get all potentially colliding blocks
    let min_block = (player_aabb.min - Vec3::ONE * 0.5).floor().as_ivec3();
    let max_block = (player_aabb.max + Vec3::ONE * 0.5).ceil().as_ivec3();

    // Check collisions with all nearby blocks
    let mut collisions = Vec::new();

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                let block_pos = Vec3::new(x as f32, y as f32, z as f32);
                if is_position_solid(&chunk_manager, block_pos) {
                    let block_aabb = Aabb::new(block_pos + Vec3::splat(0.5), Vec3::ONE);
                    if player_aabb.intersects(&block_aabb) {
                        collisions.push(block_aabb);
                    }
                }
            }
        }
    }

    // Reset ground check
    physics.on_ground = false;

    // Resolve collisions
    for block in collisions.iter() {
        let overlap = Vec3::new(
            if (new_pos.x - block.min.x).abs() < (block.max.x - new_pos.x).abs() {
                block.min.x - (new_pos.x + PLAYER_SIZE.x * 0.5)
            } else {
                block.max.x - (new_pos.x - PLAYER_SIZE.x * 0.5)
            },
            if (new_pos.y - block.min.y).abs() < (block.max.y - new_pos.y).abs() {
                block.min.y - (new_pos.y + PLAYER_SIZE.y * 0.5)
            } else {
                block.max.y - (new_pos.y - PLAYER_SIZE.y * 0.5)
            },
            if (new_pos.z - block.min.z).abs() < (block.max.z - new_pos.z).abs() {
                block.min.z - (new_pos.z + PLAYER_SIZE.z * 0.5)
            } else {
                block.max.z - (new_pos.z - PLAYER_SIZE.z * 0.5)
            },
        );

        let overlap_array = overlap.to_array();

        // Find smallest overlap axis
        let (axis, value) = overlap_array
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .unwrap();

        // Apply correction
        match axis {
            0 => {
                new_pos.x += value;
                velocity.0.x = 0.0;
            }
            1 => {
                new_pos.y += value;
                if *value > 0.0 {
                    physics.on_ground = true;
                }
                velocity.0.y = 0.0;
            }
            2 => {
                new_pos.z += value;
                velocity.0.z = 0.0;
            }
            _ => unreachable!(),
        }
    }

    // Apply gravity if not on ground
    if !physics.on_ground {
        velocity.0.y += GRAVITY * time.delta_secs();
        velocity.0.y = velocity.0.y.max(TERMINAL_VELOCITY);
    }

    // Apply final position
    transform.translation = new_pos;

    // Apply drag to horizontal velocity when on ground
    if physics.on_ground {
        const GROUND_DRAG: f32 = 0.97;
        velocity.0.x *= GROUND_DRAG;
        velocity.0.z *= GROUND_DRAG;
    }
}

// Helper function to check if a world position is inside a solid block
fn is_position_solid(chunk_manager: &ChunkManager, pos: Vec3) -> bool {
    // Convert world position to block position (using floor for negative numbers)
    let block_x = pos.x.floor();
    let block_y = pos.y.floor();
    let block_z = pos.z.floor();

    // Calculate chunk position
    let chunk_x = if block_x < 0.0 {
        ((block_x + 1.0) / 16.0).floor() as i32 - 1
    } else {
        (block_x / 16.0).floor() as i32
    };
    let chunk_y = if block_y < 0.0 {
        ((block_y + 1.0) / 16.0).floor() as i32 - 1
    } else {
        (block_y / 16.0).floor() as i32
    };
    let chunk_z = if block_z < 0.0 {
        ((block_z + 1.0) / 16.0).floor() as i32 - 1
    } else {
        (block_z / 16.0).floor() as i32
    };

    let chunk_pos = IVec3::new(chunk_x, chunk_y, chunk_z);

    if let Some(chunk) = chunk_manager.chunks.get(&chunk_pos) {
        // Calculate local position within chunk
        let local_x = ((block_x.rem_euclid(16.0)) as i32).rem_euclid(16) as u32;
        let local_y = ((block_y.rem_euclid(16.0)) as i32).rem_euclid(16) as u32;
        let local_z = ((block_z.rem_euclid(16.0)) as i32).rem_euclid(16) as u32;
        let local_pos = UVec3::new(local_x, local_y, local_z);

        chunk.get(local_pos).is_solid()
    } else {
        false // Changed to false to allow falling in void
    }
}
