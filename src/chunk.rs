use bevy::{prelude::*, utils::HashMap};

use crate::{
    block::{Block, BlockFaces},
    voxel_mesh::{VoxelFace, VoxelMesh},
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

    pub fn render(&self, chunks: &HashMap<IVec3, Chunk>, chunk_pos: IVec3) -> VoxelMesh {
        let mut mesh = VoxelMesh::new();

        for (i, block) in self.blocks.iter().enumerate() {
            if block.is_air() {
                continue;
            }

            let position = Self::from_index(i as u32);
            let faces = self.get_visible_faces(position, chunks, chunk_pos);
            block.render(&mut mesh, faces, position, self, chunks, chunk_pos);
        }

        mesh
    }

    fn get_visible_faces(
        &self,
        pos: UVec3,
        chunks: &HashMap<IVec3, Chunk>,
        chunk_pos: IVec3,
    ) -> BlockFaces {
        let mut faces = BlockFaces {
            left: true,
            right: true,
            front: true,
            back: true,
            top: true,
            bottom: true,
        };

        // Helper closure to check neighboring blocks
        let check_neighbor = |x: i32, y: i32, z: i32| -> bool {
            let (chunk_offset, block_pos) = Self::get_neighbor_positions(pos, x, y, z);
            let neighbor_chunk_pos = chunk_pos + chunk_offset;

            if let Some(neighbor_chunk) = chunks.get(&neighbor_chunk_pos) {
                neighbor_chunk.get(block_pos).is_solid()
            } else {
                false
            }
        };

        faces.left = !check_neighbor(-1, 0, 0);
        faces.right = !check_neighbor(1, 0, 0);
        faces.bottom = !check_neighbor(0, -1, 0);
        faces.top = !check_neighbor(0, 1, 0);
        faces.back = !check_neighbor(0, 0, -1);
        faces.front = !check_neighbor(0, 0, 1);

        faces
    }

    fn get_neighbor_positions(pos: UVec3, dx: i32, dy: i32, dz: i32) -> (IVec3, UVec3) {
        let mut chunk_offset = IVec3::ZERO;
        let mut block_pos = UVec3::new(pos.x as u32, pos.y as u32, pos.z as u32);

        // Handle x direction
        if (pos.x == 0 && dx < 0) || (pos.x == CHUNK_SIZE - 1 && dx > 0) {
            chunk_offset.x = dx;
            block_pos.x = if dx < 0 { CHUNK_SIZE - 1 } else { 0 };
        } else {
            block_pos.x = (pos.x as i32 + dx) as u32;
        }

        // Handle y direction
        if (pos.y == 0 && dy < 0) || (pos.y == CHUNK_SIZE - 1 && dy > 0) {
            chunk_offset.y = dy;
            block_pos.y = if dy < 0 { CHUNK_SIZE - 1 } else { 0 };
        } else {
            block_pos.y = (pos.y as i32 + dy) as u32;
        }

        // Handle z direction
        if (pos.z == 0 && dz < 0) || (pos.z == CHUNK_SIZE - 1 && dz > 0) {
            chunk_offset.z = dz;
            block_pos.z = if dz < 0 { CHUNK_SIZE - 1 } else { 0 };
        } else {
            block_pos.z = (pos.z as i32 + dz) as u32;
        }

        (chunk_offset, block_pos)
    }

    pub fn get_ao(
        &self,
        chunks: &HashMap<IVec3, Chunk>,
        chunk_pos: IVec3,
        pos: UVec3,
        face: VoxelFace,
    ) -> [u32; 4] {
        match face {
            VoxelFace::Top => {
                let pos = IVec3::new(pos.x as i32, pos.y as i32, pos.z as i32);
                let [s1, s2, s3, s4] = [
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(0, 1, 1)), // front
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(1, 1, 0)), // right
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(0, 1, -1)), // back
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(-1, 1, 0)), // left
                ];
                let [c1, c2, c3, c4] = [
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(1, 1, 1)), // front-right
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(1, 1, -1)), // back-right
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(-1, 1, -1)), // back-left
                    self.is_neighbor_solid(chunks, chunk_pos, pos + IVec3::new(-1, 1, 1)), // front-left
                ];

                [
                    Self::calculate_corner_ao(s4, s1, c4),
                    Self::calculate_corner_ao(s2, s1, c1),
                    Self::calculate_corner_ao(s2, s3, c2),
                    Self::calculate_corner_ao(s4, s3, c3),
                ]
            }
            _ => [2, 2, 2, 2],
        }
    }

    fn calculate_corner_ao(side1: bool, side2: bool, corner: bool) -> u32 {
        match (side1, side2, corner) {
            (true, true, _) => 0,
            (true, false, false) | (false, true, false) => 1,
            (false, false, true) => 1,
            (false, false, false) => 2,
            _ => 1,
        }
    }

    fn is_neighbor_solid(
        &self,
        chunks: &HashMap<IVec3, Chunk>,
        chunk_pos: IVec3,
        pos: IVec3,
    ) -> bool {
        let chunk_offset = IVec3::new(
            (pos.x / CHUNK_SIZE as i32).clamp(-1, 1),
            (pos.y / CHUNK_SIZE as i32).clamp(-1, 1),
            (pos.z / CHUNK_SIZE as i32).clamp(-1, 1),
        );

        let block_pos = UVec3::new(
            pos.x.rem_euclid(CHUNK_SIZE as i32) as u32,
            pos.y.rem_euclid(CHUNK_SIZE as i32) as u32,
            pos.z.rem_euclid(CHUNK_SIZE as i32) as u32,
        );

        let neighbor_chunk_pos = chunk_pos + chunk_offset;

        if let Some(chunk) = chunks.get(&neighbor_chunk_pos) {
            chunk.get(block_pos).is_solid()
        } else {
            false
        }
    }
}
