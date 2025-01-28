use bevy::prelude::*;

use crate::{
    block::{Block, BlockFaces},
    voxel_mesh::VoxelMesh,
};

const CHUNK_SIZE: u32 = 16;

#[derive(Debug, Clone)]
pub struct Chunk {
    blocks: Vec<Block>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: vec![Block::Air; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize],
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    fn to_index(x: u32, y: u32, z: u32) -> usize {
        (x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE) as usize
    }

    fn from_index(index: u32) -> UVec3 {
        let x = index % CHUNK_SIZE;
        let y = (index / CHUNK_SIZE) % CHUNK_SIZE;
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        UVec3::new(x, y, z)
    }

    pub fn get(&self, position: UVec3) -> Block {
        self.blocks[Self::to_index(position.x, position.y, position.z)]
    }

    pub fn set(&mut self, position: UVec3, block: Block) {
        self.blocks[Self::to_index(position.x, position.y, position.z)] = block;
    }

    pub fn render(&self) -> VoxelMesh {
        let mut mesh = VoxelMesh::new();

        for (i, block) in self.blocks.iter().enumerate() {
            let position = Self::from_index(i as u32);
            block.render(
                &mut mesh,
                BlockFaces {
                    top: true,
                    bottom: true,
                    left: true,
                    right: true,
                    front: true,
                    back: true,
                },
                position,
            );
        }

        mesh
    }
}
