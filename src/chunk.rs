use crate::{
    block::Block,
    position::{ChunkPos, LocalPos, CHUNK_INDICES},
    voxel_mesh::VoxelMesh,
    world::WorldMap,
};

#[derive(Debug, Clone)]
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

    pub fn render(&self, world: &WorldMap, chunk_pos: ChunkPos) -> VoxelMesh {
        let mut mesh = VoxelMesh::new();

        for (index, block) in self.blocks.iter().enumerate() {
            if block.is_air() {
                continue;
            }

            let block_pos = LocalPos::from_index(index).block_pos(chunk_pos);
            let faces = world.visible_faces(block_pos);
            block.render(&mut mesh, world, block_pos, faces);
        }

        mesh
    }
}
