mod assets;
mod block;
mod chunk;
mod texture_array;
mod voxel_material;
mod voxel_mesh;

use assets::VoxelAssets;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_flycam::PlayerPlugin;
use block::Block;
use chunk::Chunk;
use texture_array::create_texture_array;
use voxel_material::VoxelMaterial;

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Next,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
            PlayerPlugin,
        ))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Next)
                .load_collection::<VoxelAssets>(),
        )
        .add_systems(OnEnter(GameState::Next), setup)
        .run();
}

fn setup(
    mut commands: Commands,
    voxel_assets: Res<VoxelAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let array_texture = create_texture_array(voxel_assets.handles(), &mut images).unwrap();

    let material = materials.add(VoxelMaterial { array_texture });

    let mut chunk = Chunk::new();
    chunk.set(UVec3::new(0, 0, 0), Block::Rock);

    let mesh = chunk.render();

    commands.spawn((
        Mesh3d(meshes.add(mesh.build())),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
