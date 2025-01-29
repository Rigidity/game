use crate::position::BlockPos;
use crate::voxel_mesh::{VoxelFace, VoxelMesh};
use crate::world::WorldMap;

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

    pub fn render(
        &self,
        mesh: &mut VoxelMesh,
        world: &WorldMap,
        block_pos: BlockPos,
        faces: BlockFaces,
    ) {
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
                        let ao = world.ambient_occlusion(block_pos, face);
                        mesh.add_face(block_pos.local_pos(), face, 0, ao);
                    }
                }
            }
        }
    }
}
