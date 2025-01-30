use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, PrimitiveTopology},
        render_resource::VertexFormat,
    },
};

use crate::{
    position::{BlockPos, LocalPos},
    world::WorldMap,
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

    pub fn render_face(
        &mut self,
        world: &WorldMap,
        block_pos: BlockPos,
        face: VoxelFace,
        tex_index: u32,
    ) {
        let pos = block_pos.local_pos();
        let ao = self.ambient_occlusion(world, block_pos, face);

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

    fn ambient_occlusion(
        &self,
        world: &WorldMap,
        block_pos: BlockPos,
        face: VoxelFace,
    ) -> [u32; 4] {
        match face {
            VoxelFace::Top => {
                let top = block_pos.top();

                let [s1, s2, s3, s4] = [
                    world.block(top.back()).is_solid(),
                    world.block(top.right()).is_solid(),
                    world.block(top.front()).is_solid(),
                    world.block(top.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(top.back().left()).is_solid(),
                    world.block(top.back().right()).is_solid(),
                    world.block(top.front().right()).is_solid(),
                    world.block(top.front().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // front-left
                    calculate_corner_ao(s2, s3, c3), // front-right
                    calculate_corner_ao(s2, s1, c2), // back-right
                    calculate_corner_ao(s4, s1, c1), // back-left
                ]
            }
            VoxelFace::Bottom => {
                let bottom = block_pos.bottom();

                let [s1, s2, s3, s4] = [
                    world.block(bottom.back()).is_solid(),
                    world.block(bottom.right()).is_solid(),
                    world.block(bottom.front()).is_solid(),
                    world.block(bottom.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(bottom.back().left()).is_solid(),
                    world.block(bottom.back().right()).is_solid(),
                    world.block(bottom.front().right()).is_solid(),
                    world.block(bottom.front().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // front-left
                    calculate_corner_ao(s2, s3, c3), // front-right
                    calculate_corner_ao(s2, s1, c2), // back-right
                    calculate_corner_ao(s4, s1, c1), // back-left
                ]
            }
            VoxelFace::Left => {
                let left = block_pos.left();

                let [s1, s2, s3, s4] = [
                    world.block(left.back()).is_solid(),
                    world.block(left.top()).is_solid(),
                    world.block(left.front()).is_solid(),
                    world.block(left.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(left.bottom().back()).is_solid(),
                    world.block(left.top().back()).is_solid(),
                    world.block(left.top().front()).is_solid(),
                    world.block(left.bottom().front()).is_solid(),
                ];
                [
                    calculate_corner_ao(s2, s1, c2), // top-back
                    calculate_corner_ao(s4, s1, c1), // bottom-back
                    calculate_corner_ao(s4, s3, c4), // bottom-front
                    calculate_corner_ao(s2, s3, c3), // top-front
                ]
            }
            VoxelFace::Right => {
                let right = block_pos.right();

                let [s1, s2, s3, s4] = [
                    world.block(right.back()).is_solid(),
                    world.block(right.top()).is_solid(),
                    world.block(right.front()).is_solid(),
                    world.block(right.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(right.bottom().back()).is_solid(),
                    world.block(right.top().back()).is_solid(),
                    world.block(right.top().front()).is_solid(),
                    world.block(right.bottom().front()).is_solid(),
                ];

                [
                    calculate_corner_ao(s2, s1, c2), // top-back (TopLeft)
                    calculate_corner_ao(s4, s1, c1), // bottom-back (BottomLeft)
                    calculate_corner_ao(s4, s3, c4), // bottom-front (BottomRight)
                    calculate_corner_ao(s2, s3, c3), // top-front (TopRight)
                ]
            }
            VoxelFace::Front => {
                let front = block_pos.front();

                let [s1, s2, s3, s4] = [
                    world.block(front.bottom()).is_solid(),
                    world.block(front.right()).is_solid(),
                    world.block(front.top()).is_solid(),
                    world.block(front.left()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(front.bottom().left()).is_solid(),
                    world.block(front.bottom().right()).is_solid(),
                    world.block(front.top().right()).is_solid(),
                    world.block(front.top().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s4, s3, c4), // top-left
                    calculate_corner_ao(s4, s1, c1), // bottom-left
                    calculate_corner_ao(s2, s1, c2), // bottom-right
                    calculate_corner_ao(s2, s3, c3), // top-right
                ]
            }
            VoxelFace::Back => {
                let back = block_pos.back();

                let [s1, s2, s3, s4] = [
                    world.block(back.left()).is_solid(),
                    world.block(back.top()).is_solid(),
                    world.block(back.right()).is_solid(),
                    world.block(back.bottom()).is_solid(),
                ];
                let [c1, c2, c3, c4] = [
                    world.block(back.bottom().left()).is_solid(),
                    world.block(back.bottom().right()).is_solid(),
                    world.block(back.top().right()).is_solid(),
                    world.block(back.top().left()).is_solid(),
                ];
                [
                    calculate_corner_ao(s2, s1, c4), // top-left
                    calculate_corner_ao(s4, s1, c1), // bottom-left
                    calculate_corner_ao(s4, s3, c2), // bottom-right
                    calculate_corner_ao(s2, s3, c3), // top-right
                ]
            }
        }
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
