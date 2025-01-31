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

const TREE_HEIGHT: i32 = 12; // Tall but not gigantic
const TREE_RADIUS: i32 = 5; // Reasonable canopy size
const HOUSE_SIZE: i32 = 5;
const STRUCTURE_ATTEMPT_SPACING: i32 = 8; // Increased structure density

#[derive(Debug, Default, Clone, Resource)]
pub struct LevelGenerator {
    noise: Perlin,
}

impl LevelGenerator {
    pub fn new(seed: u32) -> Self {
        Self {
            noise: Perlin::new(seed),
        }
    }

    fn get_density_factor(&self, pos: &Vec3) -> f64 {
        let large_scale = self.noise.get([
            pos.x as f64 * 0.005,
            pos.y as f64 * 0.005,
            pos.z as f64 * 0.005,
        ]);

        large_scale * 0.5 - 0.3
    }

    fn get_terrain_density(&self, pos: &Vec3) -> f64 {
        self.noise.get([
            pos.x as f64 * 0.02,
            pos.y as f64 * 0.02,
            pos.z as f64 * 0.02,
        ])
    }

    fn get_structure_rng(&self, pos: BlockPos) -> ChaCha8Rng {
        let mut hasher = DefaultHasher::new();
        pos.hash(&mut hasher);
        let hash = hasher.finish();
        ChaCha8Rng::seed_from_u64(hash ^ self.noise.seed() as u64)
    }

    // Add this helper function to find structure origins that could affect a chunk
    fn get_structure_positions_affecting_chunk(&self, chunk_pos: ChunkPos) -> Vec<BlockPos> {
        let chunk_min = chunk_pos.world_pos();
        let chunk_max = (chunk_pos + ChunkPos::new(1, 1, 1)).world_pos();
        let structure_radius = (HOUSE_SIZE.max(TREE_HEIGHT + TREE_RADIUS) + 1) as f32;

        // Calculate the grid positions that could affect this chunk
        let min_x =
            ((chunk_min.x - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_x =
            ((chunk_max.x + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;
        let min_z =
            ((chunk_min.z - structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).floor() as i32;
        let max_z =
            ((chunk_max.z + structure_radius) / STRUCTURE_ATTEMPT_SPACING as f32).ceil() as i32;

        // Calculate Y range to check for structures that could affect this chunk
        let check_height = TREE_HEIGHT + TREE_RADIUS;
        let min_y = chunk_pos.y - (check_height / CHUNK_SIZE as i32) - 1;
        let max_y = chunk_pos.y + (check_height / CHUNK_SIZE as i32) + 1;

        let mut positions = Vec::new();

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                for z in min_z..=max_z {
                    // Get the structure origin point
                    let origin = BlockPos::new(
                        x * STRUCTURE_ATTEMPT_SPACING,
                        y * CHUNK_SIZE as i32,
                        z * STRUCTURE_ATTEMPT_SPACING,
                    );
                    positions.push(origin);
                }
            }
        }

        positions
    }

    pub fn generate_chunk(&mut self, chunk_pos: ChunkPos) -> Chunk {
        let mut chunk = Chunk::new();

        // Generate terrain first
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let local_pos = LocalPos::new(x, y, z);
                    let block_pos = local_pos.block_pos(chunk_pos);
                    let pos = block_pos.world_pos();

                    let density_factor = self.get_density_factor(&pos);
                    let terrain_density = self.get_terrain_density(&pos);

                    if terrain_density > density_factor {
                        // Check upwards until we find air to determine if we're near a surface
                        let distance_to_surface = (0..=5)
                            .find(|&d| {
                                let check_pos = (block_pos + BlockPos::Y * d).world_pos();
                                let check_density_factor = self.get_density_factor(&check_pos);
                                let check_terrain_density = self.get_terrain_density(&check_pos);
                                check_terrain_density <= check_density_factor
                            })
                            .unwrap_or(5);

                        let block_type = if distance_to_surface == 1 {
                            Block::Grass
                        } else if distance_to_surface <= 3 {
                            Block::Dirt
                        } else {
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
            let world_pos = pos.world_pos();

            // Check density at structure position
            let density_factor = self.get_density_factor(&world_pos);
            let terrain_density = self.get_terrain_density(&world_pos);

            // Only generate if we're at an air/ground boundary
            if terrain_density > density_factor {
                continue;
            }

            let below_pos = world_pos + Vec3::new(0.0, -1.0, 0.0);
            let below_density_factor = self.get_density_factor(&below_pos);
            let below_terrain_density = self.get_terrain_density(&below_pos);
            if below_terrain_density <= below_density_factor {
                continue;
            }

            // Use deterministic RNG for this position
            let mut rng = self.get_structure_rng(pos);

            // Determine structure type first, using a single random check
            let is_tree = rng.random::<f32>() < 0.9;

            // For each block position in the chunk
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let local_pos = LocalPos::new(x, y, z);
                        let block_pos = local_pos.block_pos(chunk_pos);

                        // Get the block type from the structure at this position
                        let structure_block = if is_tree {
                            self.get_tree_block(block_pos, pos, &mut rng)
                        } else {
                            self.get_house_block(block_pos, pos, &mut rng)
                        };

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

    fn get_house_block(
        &self,
        block_pos: BlockPos,
        origin: BlockPos,
        _rng: &mut ChaCha8Rng, // We won't use RNG for house generation
    ) -> Option<Block> {
        let diff = block_pos - origin;

        // Derive size deterministically from position
        let mut hasher = DefaultHasher::new();
        origin.hash(&mut hasher);
        let size = HOUSE_SIZE + (hasher.finish() % 2) as i32;
        let height = size - 1;

        // Check if the block is within house bounds
        if diff.x >= 0
            && diff.x < size
            && diff.z >= 0
            && diff.z < size
            && diff.y >= 0
            && diff.y <= height
        {
            // Walls
            if diff.y < height {
                if diff.x == 0 || diff.x == size - 1 || diff.z == 0 || diff.z == size - 1 {
                    // Door
                    if diff.y == 1 && diff.x == 0 && diff.z == size / 2 {
                        return Some(Block::Air);
                    }
                    return Some(Block::Rock);
                }
            } else {
                // Roof
                return Some(Block::Rock);
            }
        }

        None
    }
}
