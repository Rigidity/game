use bevy::prelude::*;

use crate::{
    assets::VoxelAssets,
    voxel_mesh::{VoxelFace, VoxelMesh},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockFaces {
    pub left: bool,
    pub right: bool,
    pub front: bool,
    pub back: bool,
    pub top: bool,
    pub bottom: bool,
}

impl BlockFaces {
    pub fn get(&self, face: VoxelFace) -> bool {
        match face {
            VoxelFace::Left => self.left,
            VoxelFace::Right => self.right,
            VoxelFace::Front => self.front,
            VoxelFace::Back => self.back,
            VoxelFace::Top => self.top,
            VoxelFace::Bottom => self.bottom,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air,
    Rock,
}

impl Block {
    pub fn is_air(&self) -> bool {
        matches!(self, Block::Air)
    }

    pub fn is_solid(&self) -> bool {
        !self.is_air()
    }

    pub fn render(&self, mesh: &mut VoxelMesh, faces: BlockFaces, position: UVec3) {
        match self {
            Self::Air => {}
            Self::Rock => {
                for face in [
                    VoxelFace::Left,
                    VoxelFace::Right,
                    VoxelFace::Front,
                    VoxelFace::Back,
                    VoxelFace::Top,
                    VoxelFace::Bottom,
                ] {
                    if faces.get(face) {
                        mesh.add_face(position, face, VoxelAssets::rock());
                    }
                }
            }
        }
    }
}
