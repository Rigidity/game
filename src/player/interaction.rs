use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use crate::{
    aabb::Aabb,
    block::Block,
    level::{Level, Modified},
    loader::VoxelMaterial,
    position::{BlockPos, ChunkPos, CHUNK_SIZE},
    voxel_mesh::VoxelFace,
};

use super::{Player, PlayerCamera};

const MAX_REACH: f32 = 5.0;

#[derive(Debug, Default, Clone, Copy, Resource)]
pub struct FocusedBlock {
    pub block_pos: Option<BlockPos>,
    pub air_pos: Option<BlockPos>,
    pub face: Option<VoxelFace>,
}

#[derive(Debug, Default, Clone, Resource)]
pub struct InteractionTimer(pub Timer);

pub fn update_focused_block(
    level: Res<Level>,
    mut focused_block: ResMut<FocusedBlock>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Transform, &Parent), With<PlayerCamera>>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<(&ChunkPos, &MeshMaterial3d<VoxelMaterial>)>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let Ok(window) = primary_window.get_single() else {
        return;
    };

    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    let (camera_transform, camera_parent) = camera_query.single();
    let player_transform = player_query.get(camera_parent.get()).unwrap();

    let ray_origin = player_transform.translation + camera_transform.translation;
    let ray_direction = camera_transform.forward().normalize();

    let Some((hit_pos, hit_normal)) = raycast_blocks(&level, ray_origin, ray_direction, MAX_REACH)
    else {
        focused_block.block_pos = None;
        focused_block.air_pos = None;
        focused_block.face = None;

        for (_, material) in chunk_query.iter() {
            if !materials
                .get(&material.0)
                .unwrap()
                .block_interaction
                .is_set()
            {
                continue;
            }

            materials
                .get_mut(&material.0)
                .unwrap()
                .block_interaction
                .unset();
        }

        return;
    };

    let face = if hit_normal.x < 0.0 {
        VoxelFace::Left
    } else if hit_normal.x > 0.0 {
        VoxelFace::Right
    } else if hit_normal.y < 0.0 {
        VoxelFace::Bottom
    } else if hit_normal.y > 0.0 {
        VoxelFace::Top
    } else if hit_normal.z < 0.0 {
        VoxelFace::Back
    } else {
        VoxelFace::Front
    };

    focused_block.face = Some(face);

    let block_pos = BlockPos::from_world(hit_pos);
    let air_pos = BlockPos::from_world(hit_pos + hit_normal);

    if level.block(block_pos) != Block::Air {
        focused_block.block_pos = Some(block_pos);
    } else {
        focused_block.block_pos = None;
    }

    if level.block(air_pos) == Block::Air {
        focused_block.air_pos = Some(air_pos);
    } else {
        focused_block.air_pos = None;
    }

    let hit_chunk_pos = block_pos.chunk_pos();
    let local_pos = block_pos.local_pos();

    for (&chunk_pos, material) in chunk_query.iter() {
        if chunk_pos != hit_chunk_pos
            && !materials
                .get(&material.0)
                .unwrap()
                .block_interaction
                .is_set()
        {
            continue;
        }

        let material = materials.get_mut(&material.0).unwrap();

        if chunk_pos == hit_chunk_pos && focused_block.block_pos.is_some() {
            material.block_interaction.set(local_pos, face);
        } else {
            material.block_interaction.unset();
        }
    }
}

pub fn break_or_place_block(
    time: Res<Time>,
    mouse: Res<ButtonInput<MouseButton>>,
    focused_block: Res<FocusedBlock>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut level: ResMut<Level>,
    mut timer: ResMut<InteractionTimer>,
    mut commands: Commands,
) {
    timer.0.tick(time.delta());

    let Ok(window) = primary_window.get_single() else {
        return;
    };

    if window.cursor_options.grab_mode == CursorGrabMode::None {
        return;
    }

    let is_breaking = mouse.just_pressed(MouseButton::Left)
        || (mouse.pressed(MouseButton::Left) && timer.0.just_finished());

    let is_placing = mouse.just_pressed(MouseButton::Right)
        || (mouse.pressed(MouseButton::Right) && timer.0.just_finished());

    if !is_breaking && !is_placing {
        return;
    }

    timer.0.reset();

    let block_pos = if is_breaking {
        focused_block.block_pos
    } else {
        focused_block.air_pos
    };

    let Some(block_pos) = block_pos else {
        return;
    };

    let chunk_pos = block_pos.chunk_pos();
    let local_pos = block_pos.local_pos();

    let Some(chunk) = level.chunk_mut(chunk_pos) else {
        return;
    };

    if is_breaking {
        chunk.set(local_pos, Block::Air);
    } else {
        chunk.set(local_pos, Block::Leaves);
    }

    if let Some(entity) = level.entity(chunk_pos) {
        commands.entity(entity).insert(Modified);
    }

    let neighbors = [
        ChunkPos::X,
        ChunkPos::NEG_X,
        ChunkPos::Y,
        ChunkPos::NEG_Y,
        ChunkPos::Z,
        ChunkPos::NEG_Z,
    ];

    for &offset in &neighbors {
        if local_pos.x == 0
            || local_pos.x == CHUNK_SIZE - 1
            || local_pos.y == 0
            || local_pos.y == CHUNK_SIZE - 1
            || local_pos.z == 0
            || local_pos.z == CHUNK_SIZE - 1
        {
            let neighbor_pos = chunk_pos + offset;
            if let Some(entity) = level.entity(neighbor_pos) {
                commands.entity(entity).insert(Modified);
            }
        }
    }
}

fn raycast_blocks(
    level: &Level,
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
) -> Option<(Vec3, Vec3)> {
    let mut current_pos = ray_origin;
    let step = 0.1; // Small step size for reasonable accuracy

    for _ in 0..((max_distance / step) as i32) {
        let block_pos = current_pos.floor();

        if level.block(BlockPos::from_world(block_pos)) != Block::Air {
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
