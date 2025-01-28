mod block;
mod chunk;
mod voxel_material;
mod voxel_mesh;

use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use block::Block;
use chunk::Chunk;
use noise::{NoiseFn, Perlin};
use std::collections::HashMap;
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
            PlayerPlugin,
        ))
        .insert_resource(ChunkManager {
            chunks: HashMap::new(),
        })
        .add_systems(
            Startup,
            (generate_chunks, build_chunk_meshes.after(generate_chunks)),
        )
        .run();
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
        let mesh = chunk.render(&chunk_manager.chunks, chunk_pos);

        commands.spawn((
            Mesh3d(meshes.add(mesh.build())),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(
                chunk_pos.x as f32 * 16.0,
                chunk_pos.y as f32 * 16.0,
                chunk_pos.z as f32 * 16.0,
            ),
        ));
    }
}
