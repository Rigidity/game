use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_resource::VertexFormat,
    },
};

use crate::position::LocalPos;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VoxelCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl VoxelCorner {
    fn to_index(self) -> u32 {
        match self {
            Self::TopLeft => 0,
            Self::TopRight => 1,
            Self::BottomLeft => 2,
            Self::BottomRight => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoxelFace {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}

#[derive(Debug, Default, Clone)]
pub struct VoxelMesh {
    voxels: Vec<u32>,
    indices: Vec<u32>,
    positions: Vec<Vec3>,
}

impl VoxelMesh {
    pub const VOXEL: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Voxel", 0, VertexFormat::Uint32);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex(
        &mut self,
        pos: LocalPos,
        corner: VoxelCorner,
        face: VoxelFace,
        tex_index: u32,
        ao: u32,
    ) -> u32 {
        self.positions
            .push(Vec3::new(pos.x() as f32, pos.y() as f32, pos.z() as f32));

        let x = (pos.x() as u32 & 0x0F) << 28;
        let y = (pos.y() as u32 & 0x0F) << 24;
        let z = (pos.z() as u32 & 0x0F) << 20;
        let corner = (corner.to_index()) << 18;
        let face = (face as u32) << 15;
        let ao = (ao & 0x3) << 13; // Pack AO into bits 13-14
        let tex_index = tex_index & 0x1FFF; // Reduced to 13 bits to make room for AO
        let voxel = x | y | z | corner | face | ao | tex_index;
        self.voxels.push(voxel);
        self.voxels.len() as u32 - 1
    }

    pub fn add_face(&mut self, pos: LocalPos, face: VoxelFace, tex_index: u32, ao: [u32; 4]) {
        match face {
            VoxelFace::Top => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[3]);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Bottom => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[3]);
                self.add_indices([a, c, b, a, d, c]);
            }
            VoxelFace::Left => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[3]);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Right => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[3]);
                self.add_indices([a, c, b, a, d, c]);
            }
            VoxelFace::Front => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[3]);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Back => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, tex_index, ao[0]);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, tex_index, ao[1]);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, tex_index, ao[2]);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, tex_index, ao[3]);
                self.add_indices([a, c, b, a, d, c]);
            }
        }
    }

    pub fn add_indices(&mut self, indices: impl IntoIterator<Item = u32>) {
        self.indices.extend(indices);
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        mesh.insert_attribute(Self::VOXEL, self.voxels);
        mesh.insert_indices(Indices::U32(self.indices.clone()));
        mesh
    }
}
