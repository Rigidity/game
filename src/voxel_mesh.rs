use bevy::{
    asset::RenderAssetUsages,
    math::UVec3,
    render::{
        mesh::{Indices, Mesh, MeshVertexAttribute, PrimitiveTopology},
        render_resource::VertexFormat,
    },
};

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
}

impl VoxelMesh {
    pub const VOXEL: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Voxel", 0, VertexFormat::Uint32);

    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_vertex(
        &mut self,
        pos: UVec3,
        corner: VoxelCorner,
        face: VoxelFace,
        color: u32,
    ) -> u32 {
        let x = (pos.x & 0x0F) << 28;
        let y = (pos.y & 0x0F) << 24;
        let z = (pos.z & 0x0F) << 20;
        let corner = (corner.to_index()) << 18;
        let face = (face as u32) << 15;
        let color = color & 0x7FFF;
        let voxel = x | y | z | corner | face | color;
        self.voxels.push(voxel);
        self.voxels.len() as u32 - 1
    }

    pub fn add_face(&mut self, pos: UVec3, face: VoxelFace, color: u32) {
        match face {
            VoxelFace::Top => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Bottom => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                self.add_indices([a, c, b, a, d, c]);
            }
            VoxelFace::Left => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Right => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
                self.add_indices([a, c, b, a, d, c]);
            }
            VoxelFace::Front => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
                self.add_indices([a, b, c, a, c, d]);
            }
            VoxelFace::Back => {
                let a = self.add_vertex(pos, VoxelCorner::BottomLeft, face, color);
                let b = self.add_vertex(pos, VoxelCorner::TopLeft, face, color);
                let c = self.add_vertex(pos, VoxelCorner::TopRight, face, color);
                let d = self.add_vertex(pos, VoxelCorner::BottomRight, face, color);
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
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}
