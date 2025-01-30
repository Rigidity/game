use bevy::{
    asset::RenderAssetUsages,
    image::TextureAccessError,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError, TextureDimension, TextureFormat,
        },
    },
};
use bevy_asset_loader::prelude::*;

use crate::{game_state::GameState, voxel_mesh::VoxelMesh};

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
            .add_systems(OnEnter(GameState::Setup), setup_global_voxel_material);
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
}

#[derive(Debug, Clone, Resource)]
pub struct GlobalVoxelMaterial(pub Handle<VoxelMaterial>);

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

fn setup_global_voxel_material(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let array_texture = create_texture_array(
        vec![
            image_assets.rock.clone(),
            image_assets.dirt.clone(),
            image_assets.grass_side.clone(),
            image_assets.grass.clone(),
        ],
        &mut images,
    )
    .unwrap();

    commands.insert_resource(GlobalVoxelMaterial(
        materials.add(VoxelMaterial { array_texture }),
    ));
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
        &Srgba::WHITE.to_u8_array(),
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

    Ok(images.add(array_image))
}
