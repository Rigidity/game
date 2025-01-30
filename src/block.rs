use crate::position::BlockPos;
use crate::voxel_mesh::{VoxelFace, VoxelMesh};
use crate::world::WorldMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct BlockFaces {
    pub left: bool,
    pub right: bool,
    pub front: bool,
    pub back: bool,
    pub top: bool,
    pub bottom: bool,
}

impl BlockFaces {
    pub fn get(self, face: VoxelFace) -> bool {
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
    Dirt,
    Grass,
}

impl Block {
    pub fn is_air(self) -> bool {
        matches!(self, Block::Air)
    }

    pub fn is_solid(self) -> bool {
        !self.is_air()
    }

    pub fn render(
        self,
        mesh: &mut VoxelMesh,
        world: &WorldMap,
        block_pos: BlockPos,
        faces: BlockFaces,
    ) {
        for face in [
            VoxelFace::Left,
            VoxelFace::Right,
            VoxelFace::Front,
            VoxelFace::Back,
            VoxelFace::Top,
            VoxelFace::Bottom,
        ] {
            if faces.get(face) {
                mesh.render_face(
                    world,
                    block_pos,
                    face,
                    #[allow(clippy::match_same_arms)]
                    match (self, face) {
                        (Self::Air, _) => continue,
                        (Self::Rock, _) => 0,
                        (Self::Dirt, _) => 1,
                        (Self::Grass, VoxelFace::Top) => 3,
                        (Self::Grass, VoxelFace::Bottom) => 1,
                        (Self::Grass, _) => 2,
                    },
                );
            }
        }
    }
}
