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

    pub fn add_vertex(&mut self, pos: UVec3, corner: VoxelCorner, texture_index: u32) -> u32 {
        let x = (pos.x & 0x0F) << 28;
        let y = (pos.y & 0x0F) << 24;
        let z = (pos.z & 0x0F) << 20;
        let corner = (corner.to_index()) << 18;
        let texture_index = texture_index & 0x3FFFF;
        let voxel = x | y | z | corner | texture_index;
        self.voxels.push(voxel);
        self.voxels.len() as u32 - 1
    }

    pub fn add_face(&mut self, pos: UVec3, face: VoxelFace, texture_index: u32) {
        if face != VoxelFace::Top {
            return;
        }

        match face {
            VoxelFace::Left => {
                let a = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z + 1),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                self.add_indices([a, d, c, c, b, a]);
            }
            VoxelFace::Right => {
                let a = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z + 1),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                self.add_indices([a, b, c, c, d, a]);
            }
            VoxelFace::Top => {
                let a = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                self.add_indices([a, d, c, c, b, a]);
            }
            VoxelFace::Bottom => {
                let a = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z + 1),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z + 1),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                self.add_indices([a, b, c, c, d, a]);
            }
            VoxelFace::Front => {
                let a = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z + 1),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z + 1),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z + 1),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                self.add_indices([a, b, c, c, d, a]);
            }
            VoxelFace::Back => {
                let a = self.add_vertex(
                    UVec3::new(pos.x, pos.y, pos.z),
                    VoxelCorner::BottomLeft,
                    texture_index,
                );
                let b = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y, pos.z),
                    VoxelCorner::BottomRight,
                    texture_index,
                );
                let c = self.add_vertex(
                    UVec3::new(pos.x + 1, pos.y + 1, pos.z),
                    VoxelCorner::TopRight,
                    texture_index,
                );
                let d = self.add_vertex(
                    UVec3::new(pos.x, pos.y + 1, pos.z),
                    VoxelCorner::TopLeft,
                    texture_index,
                );
                self.add_indices([a, d, c, c, b, a]);
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
