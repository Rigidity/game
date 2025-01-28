mod block;
mod chunk;
mod voxel_material;
mod voxel_mesh;

use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use block::Block;
use chunk::Chunk;
use noise::{NoiseFn, Perlin};
use voxel_material::VoxelMaterial;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
            PlayerPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let material = materials.add(VoxelMaterial {});
    let perlin = Perlin::new(42);

    // Generate a 4x4x4 grid of chunks
    for chunk_x in 0..4 {
        for chunk_y in 0..4 {
            for chunk_z in 0..4 {
                let mut chunk = Chunk::new();

                // Fill each 16x16x16 chunk
                for x in 0..16 {
                    for y in 0..16 {
                        for z in 0..16 {
                            // Calculate world position for noise
                            let world_x = chunk_x * 16 + x;
                            let world_y = chunk_y * 16 + y;
                            let world_z = chunk_z * 16 + z;

                            // Get noise value at this position
                            let noise_value = perlin.get([
                                world_x as f64 * 0.1, // Reduced scale for smoother terrain
                                world_y as f64 * 0.1,
                                world_z as f64 * 0.1,
                            ]);

                            let normalized_noise = (noise_value + 1.0) / 2.0;

                            // Use height (y) for terrain generation
                            let height_threshold = (world_y as f64 / (4.0 * 16.0)) * 0.8;

                            if normalized_noise > height_threshold {
                                chunk.set(UVec3::new(x, y, z), Block::Rock);
                            }
                        }
                    }
                }

                let mesh = chunk.render();

                commands.spawn((
                    Mesh3d(meshes.add(mesh.build())),
                    MeshMaterial3d(material.clone()),
                    Transform::from_xyz(
                        chunk_x as f32 * 16.0,
                        chunk_y as f32 * 16.0,
                        chunk_z as f32 * 16.0,
                    ),
                ));
            }
        }
    }
}
