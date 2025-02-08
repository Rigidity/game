use std::hash::{DefaultHasher, Hash, Hasher};

use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    block::Block,
    chunk::Chunk,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_SIZE},
};

const TREE_HEIGHT: i32 = 6; // Tall but not gigantic
const TREE_RADIUS: i32 = 5; // Reasonable canopy size
const STRUCTURE_ATTEMPT_SPACING: i32 = 10; // Closer base spacing

#[derive(Debug, Default, Clone, Resource)]
pub struct LevelGenerator {
    density_noise: Perlin,
    terrain_noise: Perlin,
    moisture_noise: Perlin,
}

impl LevelGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            density_noise: Perlin::new(seed),
            terrain_noise: Perlin::new(seed + 1),
            moisture_noise: Perlin::new(seed + 2),
        }
    }

    fn get_moisture(&self, pos: &Vec3) -> f64 {
        (self.moisture_noise.get([
            pos.x as f64 * 0.003,
            pos.y as f64 * 0.01,
            pos.z as f64 * 0.003,
        ]) + 1.0)
            / 2.0
    }

    fn get_structure_rng(&self, pos: BlockPos) -> ChaCha8Rng {
        let mut hasher = DefaultHasher::new();
        pos.hash(&mut hasher);
        let hash = hasher.finish();
        ChaCha8Rng::seed_from_u64(hash ^ self.density_noise.seed() as u64)
    }

    fn get_height(&self, x: f64, z: f64) -> i32 {
        // Create a more dramatic heightmap using our terrain noise
        let base_height = self.terrain_noise.get([x * 0.01, z * 0.01, 0.0]);

        // Add medium-scale variation
        let medium_detail = self.terrain_noise.get([x * 0.05, z * 0.05, 0.0]) * 0.3;

        // Add small-scale detail
        let small_detail = self.terrain_noise.get([x * 0.1, z * 0.1, 0.0]) * 0.1;

        // Combine and amplify the height variations
        // Multiply by 16 instead of 4 for more dramatic heights
        ((base_height + medium_detail + small_detail) * 16.0).round() as i32
    }

    pub fn generate_chunk(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();

        // Generate terrain
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let local_pos = LocalPos::new(x, y, z);
                    let block_pos = local_pos.block_pos(chunk_pos);
                    let world_pos = block_pos.world_pos();
                    let height = self.get_height(world_pos.x as f64, world_pos.z as f64);

                    if block_pos.y <= height {
                        let moisture = self.get_moisture(&world_pos);

                        let block_type = if block_pos.y == height {
                            // Surface layer
                            if moisture > 0.3 {
                                let gravel_noise = self.terrain_noise.get([
                                    world_pos.x as f64 * 0.08,
                                    world_pos.z as f64 * 0.08,
                                    0.0,
                                ]);

                                if gravel_noise > 0.6 {
                                    Block::Gravel
                                } else {
                                    Block::Grass
                                }
                            } else {
                                Block::Sand
                            }
                        } else if block_pos.y >= height - 2 {
                            // Subsurface layers
                            if moisture > 0.3 {
                                Block::Dirt
                            } else {
                                Block::Sand
                            }
                        } else {
                            // Deep layers
                            Block::Rock
                        };

                        chunk.set(local_pos, block_type);
                    }
                }
            }
        }

        // Get all possible structure positions that could affect this chunk
        let structure_positions = self.get_structure_positions_affecting_chunk(chunk_pos);

        // Try generating structures at each position
        for pos in structure_positions {
            // Get the height at this position
            let height = self.get_height(pos.x as f64, pos.z as f64);
            let valid_ground = pos.y == height + 1; // Check if we're one block above the surface

            if !valid_ground {
                continue;
            }

            // Use deterministic RNG for this position
            let mut rng = self.get_structure_rng(pos);

            // For each block position in the chunk
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let local_pos = LocalPos::new(x, y, z);
                        let block_pos = local_pos.block_pos(chunk_pos);

                        // Get the block type from the structure at this position
                        let structure_block = self.get_tree_block(block_pos, pos, &mut rng);

                        // If the structure defines a block here, override the terrain
                        if let Some(block) = structure_block {
                            chunk.set(local_pos, block);
                        }
                    }
                }
            }
        }

        chunk
    }

    fn get_structure_positions_affecting_chunk(&self, chunk_pos: ChunkPos) -> Vec<BlockPos> {
        let chunk_min = chunk_pos.world_pos();
        let chunk_max = (chunk_pos + ChunkPos::new(1, 1, 1)).world_pos();
        let structure_radius = (0.max(TREE_HEIGHT + TREE_RADIUS) + 1) as f32;

        let min_x =
            ((chunk_min.x - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_x =
            ((chunk_max.x + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;
        let min_z =
            ((chunk_min.z - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_z =
            ((chunk_max.z + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;

        let mut positions = Vec::new();

        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let base_x = x * STRUCTURE_ATTEMPT_SPACING;
                let base_z = z * STRUCTURE_ATTEMPT_SPACING;

                // Get the height at this position
                let height = self.get_height(base_x as f64, base_z as f64);

                let base_pos = BlockPos::new(
                    base_x,
                    height + 1, // One block above surface
                    base_z,
                );

                let mut cell_rng = self.get_structure_rng(base_pos);

                if cell_rng.random::<f32>() < 0.1 {
                    let offset_x = cell_rng.random_range(-2..3);
                    let offset_z = cell_rng.random_range(-2..3);

                    // Get height at the offset position
                    let final_x = base_x + offset_x;
                    let final_z = base_z + offset_z;
                    let final_height = self.get_height(final_x as f64, final_z as f64);

                    let origin = BlockPos::new(final_x, final_height + 1, final_z);
                    positions.push(origin);
                }
            }
        }

        positions
    }

    fn get_tree_block(
        &self,
        block_pos: BlockPos,
        origin: BlockPos,
        rng: &mut ChaCha8Rng,
    ) -> Option<Block> {
        let diff = block_pos - origin;

        // Derive height deterministically from position
        let mut hasher = DefaultHasher::new();
        origin.hash(&mut hasher);
        let height = TREE_HEIGHT + (hasher.finish() % 3) as i32;

        // Trunk
        if diff.x == 0 && diff.z == 0 && diff.y >= 0 && diff.y < height {
            return Some(Block::Wood);
        }

        // Leaves
        let leaf_start = height - 4;
        let leaf_height = 5;

        if diff.y >= leaf_start && diff.y < leaf_start + leaf_height {
            let y_level = diff.y - leaf_start;
            let radius = if y_level == 0 || y_level == leaf_height - 1 {
                1
            } else {
                2
            };

            if diff.x.abs() <= radius && diff.z.abs() <= radius {
                // Skip corners for rounder appearance
                if diff.x * diff.x + diff.z * diff.z <= radius * radius + 1 {
                    // Use RNG only for leaf detail variation
                    if (diff.x.abs() == radius || diff.z.abs() == radius)
                        && rng.random::<f32>() < 0.5
                    {
                        return None;
                    }
                    return Some(Block::Leaves);
                }
            }
        }

        None
    }
}
