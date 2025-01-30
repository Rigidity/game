use serde::{Deserialize, Serialize};

use crate::{
    block::{Block, BlockFaces},
    level::Level,
    position::{BlockPos, ChunkPos, LocalPos, CHUNK_INDICES},
    voxel_mesh::VoxelMesh,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    blocks: Vec<Block>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: vec![Block::Air; CHUNK_INDICES],
        }
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, pos: LocalPos) -> Block {
        self.blocks[pos.index()]
    }

    pub fn set(&mut self, pos: LocalPos, block: Block) {
        self.blocks[pos.index()] = block;
    }

    pub fn render(&self, level: &Level, chunk_pos: ChunkPos) -> VoxelMesh {
        let mut mesh = VoxelMesh::new();

        for (index, block) in self.blocks.iter().enumerate() {
            if block == &Block::Air {
                continue;
            }

            let block_pos = LocalPos::from_index(index).block_pos(chunk_pos);
            let faces = visible_faces(level, block_pos);
            block.render(&mut mesh, level, block_pos, faces);
        }

        mesh
    }
}

fn visible_faces(level: &Level, pos: BlockPos) -> BlockFaces {
    BlockFaces {
        left: !level.block(pos.left()).is_solid(),
        right: !level.block(pos.right()).is_solid(),
        front: !level.block(pos.front()).is_solid(),
        back: !level.block(pos.back()).is_solid(),
        top: !level.block(pos.top()).is_solid(),
        bottom: !level.block(pos.bottom()).is_solid(),
    }
}
