use bevy::prelude::*;

use crate::{aabb::Aabb, game_state::GameState, player::Player, ChunkManager};

#[derive(Debug, Clone, Copy)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_physics.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Velocity(pub Vec3);

fn apply_physics(
    time: Res<Time>,
    chunk_manager: Res<ChunkManager>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Player)>,
) {
    let (mut transform, mut velocity, mut physics) = query.single_mut();

    const GRAVITY: f32 = -25.0;
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
pub fn is_position_solid(chunk_manager: &ChunkManager, pos: Vec3) -> bool {
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
