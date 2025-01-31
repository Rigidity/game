use bevy::{
    asset::RenderAssetUsages,
    image::TextureAccessError,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderRef, ShaderType,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat,
        },
    },
};
use bevy_asset_loader::prelude::*;

use crate::{
    game_state::GameState,
    position::LocalPos,
    voxel_mesh::{VoxelFace, VoxelMesh},
};

#[derive(Debug, Clone, Copy)]
pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default())
            .init_state::<GameState>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Setup)
                    .load_collection::<ImageAssets>(),
            )
            .add_systems(OnEnter(GameState::Setup), setup_global_texture_array);
    }
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "Voxels/Rock.png")]
    pub rock: Handle<Image>,
    #[asset(path = "Voxels/Dirt.png")]
    pub dirt: Handle<Image>,
    #[asset(path = "Voxels/GrassSide.png")]
    pub grass_side: Handle<Image>,
    #[asset(path = "Voxels/Grass.png")]
    pub grass: Handle<Image>,
    #[asset(path = "Voxels/Leaves.png")]
    pub leaves: Handle<Image>,
    #[asset(path = "Voxels/Wood.png")]
    pub wood: Handle<Image>,
    #[asset(path = "Voxels/WoodSide.png")]
    pub wood_side: Handle<Image>,
}

#[derive(Debug, Clone, Resource)]
pub struct GlobalTextureArray(pub Handle<Image>);

#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct BlockInteraction {
    x: u32,
    y: u32,
    z: u32,
    face: u32,
    value: u32,
}

impl BlockInteraction {
    pub fn set(&mut self, pos: LocalPos, face: VoxelFace) {
        self.x = pos.x as u32;
        self.y = pos.y as u32;
        self.z = pos.z as u32;
        self.face = face as u32;
        self.value = 1;
    }

    pub fn unset(&mut self) {
        self.value = 0;
    }

    pub fn is_set(&self) -> bool {
        self.value > 0
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub array_texture: Handle<Image>,
    #[uniform(2)]
    pub block_interaction: BlockInteraction,
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/voxel.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
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

        // Basic transparency setup
        if let Some(target) = &mut descriptor.fragment.as_mut().unwrap().targets[0] {
            target.blend = Some(bevy::render::render_resource::BlendState::ALPHA_BLENDING);
            target.write_mask = bevy::render::render_resource::ColorWrites::ALL;
        }

        if let Some(depth_stencil) = &mut descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = true;
            depth_stencil.depth_compare =
                bevy::render::render_resource::CompareFunction::GreaterEqual;
        }

        Ok(())
    }
}

fn setup_global_texture_array(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    let array_texture = create_texture_array(
        vec![
            image_assets.rock.clone(),
            image_assets.dirt.clone(),
            image_assets.grass_side.clone(),
            image_assets.grass.clone(),
            image_assets.leaves.clone(),
            image_assets.wood_side.clone(),
            image_assets.wood.clone(),
        ],
        &mut images,
    )
    .unwrap();

    commands.insert_resource(GlobalTextureArray(array_texture));
}

fn create_texture_array(
    handles: Vec<Handle<Image>>,
    images: &mut Assets<Image>,
) -> Result<Handle<Image>, TextureAccessError> {
    let mut array_image = Image::new_fill(
        Extent3d {
            width: 16,
            height: 2048 * 16,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::all(),
    );

    for (i, handle) in handles.iter().enumerate() {
        let image = images.get(handle).unwrap();

        for x in 0..16 {
            for y in 0..16 {
                array_image.set_color_at(x, y + i as u32 * 16, image.get_color_at(x, y)?)?;
            }
        }
    }

    array_image.reinterpret_stacked_2d_as_array(2048);

    array_image.texture_descriptor.usage =
        bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
            | bevy::render::render_resource::TextureUsages::COPY_DST;

    Ok(images.add(array_image))
}
