use crate::{aabb::Aabb, game_state::GameState, level::Level, player::Player, position::BlockPos};
use bevy::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_physics.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Velocity(pub Vec3);

const GRAVITY: f32 = -24.0;
const TERMINAL_VELOCITY: f32 = -78.4;
const PLAYER_SIZE: Vec3 = Vec3::new(0.6, 1.8, 0.6);
const GROUND_DRAG: f32 = 0.91;
const AIR_DRAG: f32 = 0.98;

/// Get all blocks the expanded AABB could collide with
fn get_potential_collisions(level: &Level, aabb: &Aabb) -> Vec<Aabb> {
    let min_block = (aabb.min - Vec3::ONE * 0.5).floor().as_ivec3();
    let max_block = (aabb.max + Vec3::ONE * 0.5).ceil().as_ivec3();

    let mut collisions = Vec::new();

    for x in min_block.x..=max_block.x {
        for y in min_block.y..=max_block.y {
            for z in min_block.z..=max_block.z {
                let block_pos = BlockPos::new(x, y, z);
                if level.block(block_pos).is_solid() {
                    collisions.push(Aabb::new(block_pos.center(), Vec3::ONE));
                }
            }
        }
    }

    collisions
}

fn check_overlap_except(current: &Aabb, other: &Aabb, axis: usize) -> bool {
    match axis {
        1 => {
            // Y axis (check X and Z)
            current.max.x > other.min.x
                && current.min.x < other.max.x
                && current.max.z > other.min.z
                && current.min.z < other.max.z
        }
        0 => {
            // X axis (check Y and Z)
            current.max.y > other.min.y
                && current.min.y < other.max.y
                && current.max.z > other.min.z
                && current.min.z < other.max.z
        }
        2 => {
            // Z axis (check X and Y)
            current.max.x > other.min.x
                && current.min.x < other.max.x
                && current.max.y > other.min.y
                && current.min.y < other.max.y
        }
        _ => false,
    }
}

fn clip_axis(current_aabb: &mut Aabb, blocks: &[Aabb], delta: f32, axis: usize) -> f32 {
    // Do continuous collision check for any fast movement
    if delta.abs() > 1.0 {
        let steps = delta.abs().ceil() as i32;
        let step_size = delta / steps as f32;

        for _ in 0..steps {
            let small_delta = clip_axis_internal(current_aabb, blocks, step_size, axis);
            if small_delta != step_size {
                return small_delta;
            }
            // Move AABB for next iteration
            match axis {
                0 => current_aabb.translate(Vec3::new(small_delta, 0.0, 0.0)),
                1 => current_aabb.translate(Vec3::new(0.0, small_delta, 0.0)),
                2 => current_aabb.translate(Vec3::new(0.0, 0.0, small_delta)),
                _ => {}
            }
        }
        return delta;
    }

    clip_axis_internal(current_aabb, blocks, delta, axis)
}

fn clip_axis_internal(current_aabb: &Aabb, blocks: &[Aabb], mut delta: f32, axis: usize) -> f32 {
    for block in blocks {
        // Only check collision if we overlap on other axes
        if !check_overlap_except(current_aabb, block, axis) {
            continue;
        }

        let (min, max) = match axis {
            0 => (current_aabb.min.x, current_aabb.max.x),
            1 => (current_aabb.min.y, current_aabb.max.y),
            2 => (current_aabb.min.z, current_aabb.max.z),
            _ => continue,
        };

        let (block_min, block_max) = match axis {
            0 => (block.min.x, block.max.x),
            1 => (block.min.y, block.max.y),
            2 => (block.min.z, block.max.z),
            _ => continue,
        };

        // Moving in positive direction
        if delta > 0.0 && max <= block_min {
            // Don't let movement take us inside the block
            let clip = block_min - max - 0.001;
            if clip < delta {
                delta = clip;
            }
        }
        // Moving in negative direction
        if delta < 0.0 && min >= block_max {
            // Don't let movement take us inside the block
            let clip = block_max - min + 0.001;
            if clip > delta {
                delta = clip;
            }
        }
    }

    delta
}

fn apply_physics(
    time: Res<Time>,
    level: Res<Level>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Player)>,
) {
    let (mut transform, mut velocity, mut player) = query.single_mut();
    let dt = time.delta_secs();

    // Calculate movement for this frame
    let orig_movement = velocity.0 * dt;
    let mut movement = orig_movement;

    // Create player AABB
    let mut player_aabb = Aabb::new(transform.translation, PLAYER_SIZE);

    // Get potential collisions
    let blocks = get_potential_collisions(&level, &player_aabb);

    // Y movement first
    movement.y = clip_axis(&mut player_aabb, &blocks, movement.y, 1);
    player_aabb.translate(Vec3::new(0.0, movement.y, 0.0));

    // Then X
    movement.x = clip_axis(&mut player_aabb, &blocks, movement.x, 0);
    player_aabb.translate(Vec3::new(movement.x, 0.0, 0.0));

    // Finally Z
    movement.z = clip_axis(&mut player_aabb, &blocks, movement.z, 2);
    player_aabb.translate(Vec3::new(0.0, 0.0, movement.z));

    // Update ground state based on blocked downward movement
    player.on_ground = orig_movement.y != movement.y && orig_movement.y < 0.0;

    // Zero velocities that got clipped
    if orig_movement.x != movement.x {
        velocity.0.x = 0.0;
    }
    if orig_movement.y != movement.y {
        velocity.0.y = 0.0;
    }
    if orig_movement.z != movement.z {
        velocity.0.z = 0.0;
    }

    // Apply gravity if not on ground
    if !player.on_ground {
        velocity.0.y += GRAVITY * dt;
        velocity.0.y = velocity.0.y.max(TERMINAL_VELOCITY);
    }

    // Update transform position
    transform.translation = player_aabb.center();

    // Apply drag
    if player.on_ground {
        velocity.0.x *= GROUND_DRAG;
        velocity.0.z *= GROUND_DRAG;
    } else {
        velocity.0.x *= AIR_DRAG;
        velocity.0.z *= AIR_DRAG;
    }
}
