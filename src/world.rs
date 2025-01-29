use bevy::{prelude::*, utils::HashMap};
use noise::{NoiseFn, Perlin};

use crate::{
    block::Block, chunk::Chunk, game_state::GameState, texture_array::create_texture_array,
    voxel_material::VoxelMaterial, ImageAssets, VoxelMaterials,
};

#[derive(Debug, Clone, Copy)]
pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (generate_chunks, build_chunk_meshes).chain(),
        );
    }
}

#[derive(Resource)]
pub struct WorldMap {
    pub chunks: HashMap<IVec3, Chunk>,
    pub noise: Perlin,
}

fn generate_chunks(mut world: ResMut<WorldMap>) {
    // Generate a grid of chunks
    for chunk_x in 0..16 {
        for chunk_y in 0..16 {
            for chunk_z in 0..16 {
                let mut chunk = Chunk::new();
                let chunk_pos = IVec3::new(chunk_x, chunk_y, chunk_z);

                // Fill each 16x16x16 chunk
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            let world_x = chunk_x * 16 + x as i32;
                            let world_y = chunk_y * 16 + y as i32;
                            let world_z = chunk_z * 16 + z as i32;

                            let noise_value = world.noise.get([
                                world_x as f64 * 0.04,
                                world_y as f64 * 0.04,
                                world_z as f64 * 0.04,
                            ]);

                            let normalized_noise = (noise_value + 1.0) / 2.0;
                            let height_threshold = (world_y as f64 / (4.0 * 16.0)) * 0.8;

                            if normalized_noise > height_threshold {
                                chunk.set(UVec3::new(x, y, z), Block::Rock);
                            }
                        }
                    }
                }

                world.chunks.insert(chunk_pos, chunk);
            }
        }
    }
}

fn build_chunk_meshes(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut world: ResMut<WorldMap>,
) {
    let array_texture = create_texture_array(vec![image_assets.rock.clone()], &mut images).unwrap();
    let material = materials.add(VoxelMaterial { array_texture });

    commands.insert_resource(VoxelMaterials {
        material: material.clone(),
    });

    // Clone the chunks map to avoid borrow conflict
    let chunks = world.chunks.clone();

    for (&chunk_pos, chunk) in world.chunks.iter_mut() {
        let mesh = chunk.render(&chunks, chunk_pos).build();

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                Transform::from_xyz(
                    chunk_pos.x as f32 * 16.0,
                    chunk_pos.y as f32 * 16.0,
                    chunk_pos.z as f32 * 16.0,
                ),
            ))
            .id();

        chunk.mesh_entity = Some(entity);
    }
}

pub fn regenerate_chunk_mesh(
    commands: &mut Commands,
    chunks: &mut HashMap<IVec3, Chunk>,
    chunk_pos: IVec3,
    meshes: &mut Assets<Mesh>,
    materials: &VoxelMaterials,
) {
    let chunk_cache = chunks.clone();

    if let Some(chunk) = chunks.get_mut(&chunk_pos) {
        let mesh = chunk.render(&chunk_cache, chunk_pos).build();

        if let Some(entity) = chunk.mesh_entity {
            commands.entity(entity).despawn();
        }

        chunk.mesh_entity = Some(
            commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(materials.material.clone()),
                    Transform::from_xyz(
                        chunk_pos.x as f32 * 16.0,
                        chunk_pos.y as f32 * 16.0,
                        chunk_pos.z as f32 * 16.0,
                    ),
                ))
                .id(),
        );
    }
}
