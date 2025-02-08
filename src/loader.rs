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
    ui::ItemImageCache,
    voxel_mesh::{VoxelFace, VoxelMesh},
};

#[derive(Debug, Clone, Copy)]
pub struct LoaderPlugin;

impl Plugin for LoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default())
            .init_resource::<ItemImageCache>()
            .add_loading_state(
                LoadingState::new(GameState::Loading)
                    .continue_to_state(GameState::Setup)
                    .load_collection::<BlockImages>()
                    .load_collection::<DestroyImages>()
                    .load_collection::<ItemImages>(),
            )
            .add_systems(OnEnter(GameState::Setup), setup_global_texture_array);
    }
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct BlockImages {
    #[asset(path = "Voxels/Blocks - Rock.png")]
    pub rock: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Dirt.png")]
    pub dirt: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Grass Side.png")]
    pub grass_side: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Grass.png")]
    pub grass: Handle<Image>,

    #[asset(path = "Voxels/Leaves.png")]
    pub leaves: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Wood.png")]
    pub wood: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Wood Side.png")]
    pub wood_side: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Sand.png")]
    pub sand: Handle<Image>,

    #[asset(path = "Voxels/Water.png")]
    pub water: Handle<Image>,

    #[asset(path = "Voxels/Blocks - Gravel.png")]
    pub gravel: Handle<Image>,
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct DestroyImages {
    #[asset(path = "Destroy/stage_0.png")]
    pub stage_0: Handle<Image>,

    #[asset(path = "Destroy/stage_1.png")]
    pub stage_1: Handle<Image>,

    #[asset(path = "Destroy/stage_2.png")]
    pub stage_2: Handle<Image>,

    #[asset(path = "Destroy/stage_3.png")]
    pub stage_3: Handle<Image>,

    #[asset(path = "Destroy/stage_4.png")]
    pub stage_4: Handle<Image>,

    #[asset(path = "Destroy/stage_5.png")]
    pub stage_5: Handle<Image>,

    #[asset(path = "Destroy/stage_6.png")]
    pub stage_6: Handle<Image>,

    #[asset(path = "Destroy/stage_7.png")]
    pub stage_7: Handle<Image>,

    #[asset(path = "Destroy/stage_8.png")]
    pub stage_8: Handle<Image>,

    #[asset(path = "Destroy/stage_9.png")]
    pub stage_9: Handle<Image>,
}

#[derive(Debug, Clone, AssetCollection, Resource)]
pub struct ItemImages {
    #[asset(path = "Items/Items - Binding.png")]
    pub binding: Handle<Image>,

    #[asset(path = "Items/Items - Flint.png")]
    pub flint: Handle<Image>,

    #[asset(path = "Items/Items - Handle.png")]
    pub handle: Handle<Image>,

    #[asset(path = "Items/Items - Pickaxe Binding Layer.png")]
    pub pickaxe_binding_layer: Handle<Image>,

    #[asset(path = "Items/Items - Pickaxe Handle Layer.png")]
    pub pickaxe_handle_layer: Handle<Image>,

    #[asset(path = "Items/Items - Pickaxe Head Layer.png")]
    pub pickaxe_head_layer: Handle<Image>,

    #[asset(path = "Items/Items - Pickaxe Head.png")]
    pub pickaxe_head: Handle<Image>,

    #[asset(path = "Items/Items - Plant Fiber.png")]
    pub plant_fiber: Handle<Image>,

    #[asset(path = "Items/Items - Soil.png")]
    pub soil: Handle<Image>,

    #[asset(path = "Items/Items - Twig.png")]
    pub twig: Handle<Image>,

    #[asset(path = "Items/Items - Small Bottle.png")]
    pub small_bottle: Handle<Image>,

    #[asset(path = "Items/Items - Medium Bottle.png")]
    pub medium_bottle: Handle<Image>,

    #[asset(path = "Items/Items - Large Bottle.png")]
    pub large_bottle: Handle<Image>,

    #[asset(path = "Items/Items - Glass.png")]
    pub glass: Handle<Image>,
}

impl BlockImages {
    pub fn handles(&self) -> Vec<Handle<Image>> {
        vec![
            self.rock.clone(),
            self.dirt.clone(),
            self.grass_side.clone(),
            self.grass.clone(),
            self.leaves.clone(),
            self.wood_side.clone(),
            self.wood.clone(),
            self.sand.clone(),
            self.water.clone(),
            self.gravel.clone(),
        ]
    }
}

impl DestroyImages {
    pub fn handles(&self) -> Vec<Handle<Image>> {
        vec![
            self.stage_0.clone(),
            self.stage_1.clone(),
            self.stage_2.clone(),
            self.stage_3.clone(),
            self.stage_4.clone(),
            self.stage_5.clone(),
            self.stage_6.clone(),
            self.stage_7.clone(),
            self.stage_8.clone(),
            self.stage_9.clone(),
        ]
    }
}

#[derive(Debug, Clone, Resource)]
pub struct GlobalTextureArray {
    pub textures: Handle<Image>,
    pub destroy: Handle<Image>,
}

#[derive(Debug, Default, Clone, Copy, ShaderType)]
pub struct BlockInteraction {
    x: u32,
    y: u32,
    z: u32,
    face: u32,
    value: u32,
}

impl BlockInteraction {
    pub fn set(&mut self, pos: LocalPos, face: VoxelFace, value: u32) {
        self.x = pos.x as u32;
        self.y = pos.y as u32;
        self.z = pos.z as u32;
        self.face = face as u32;
        self.value = value;
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
    #[texture(2, dimension = "2d_array")]
    #[sampler(3)]
    pub destroy_texture: Handle<Image>,
    #[uniform(4)]
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
    block_images: Res<BlockImages>,
    destroy_images: Res<DestroyImages>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.insert_resource(GlobalTextureArray {
        textures: create_texture_array(block_images.handles(), &mut images).unwrap(),
        destroy: create_texture_array(destroy_images.handles(), &mut images).unwrap(),
    });
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
