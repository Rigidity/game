use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

use crate::voxel_mesh::VoxelMesh;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout
            .0
            .get_layout(&[VoxelMesh::VOXEL.at_shader_location(0)])?;

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}
