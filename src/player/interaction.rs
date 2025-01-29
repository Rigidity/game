use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    aabb::Aabb,
    block::Block,
    position::{BlockPos, ChunkPos, CHUNK_SIZE},
    world::{regenerate_chunk_mesh, WorldMap},
    VoxelMaterials,
};

use super::{Player, PlayerCamera};

const MAX_REACH: f32 = 5.0;

pub fn handle_block_interaction(
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
        let world_pos = if mouse.just_pressed(MouseButton::Right) {
            hit_pos + hit_normal
        } else {
            hit_pos
        };

        let block_pos = BlockPos::from_world(world_pos);
        let chunk_pos = block_pos.chunk_pos();
        let local_pos = block_pos.local_pos();

        // Modify the block
        if let Some(chunk) = world.chunk_mut(chunk_pos) {
            if mouse.just_pressed(MouseButton::Left) {
                chunk.set(local_pos, Block::Air);
            } else {
                chunk.set(local_pos, Block::Rock);
            }

            // Regenerate mesh for the modified chunk
            regenerate_chunk_mesh(
                &mut commands,
                &mut world,
                chunk_pos,
                &mut meshes,
                &materials,
            );

            // Regenerate neighboring chunks if the modified block was on the edge
            let neighbors = [
                ChunkPos::X,
                ChunkPos::NEG_X,
                ChunkPos::Y,
                ChunkPos::NEG_Y,
                ChunkPos::Z,
                ChunkPos::NEG_Z,
            ];

            for &offset in &neighbors {
                if local_pos.x() == 0
                    || local_pos.x() == CHUNK_SIZE as usize - 1
                    || local_pos.y() == 0
                    || local_pos.y() == CHUNK_SIZE as usize - 1
                    || local_pos.z() == 0
                    || local_pos.z() == CHUNK_SIZE as usize - 1
                {
                    let neighbor_pos = chunk_pos + offset;
                    if world.chunk(neighbor_pos).is_some() {
                        regenerate_chunk_mesh(
                            &mut commands,
                            &mut world,
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

        if world.block(BlockPos::from_world(block_pos)).is_solid() {
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
