mod texture_array;
mod voxel_material;
mod voxel_mesh;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_flycam::PlayerPlugin;
use texture_array::create_texture_array;
use voxel_material::VoxelMaterial;
use voxel_mesh::{VoxelFace, VoxelMesh};

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Next,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "Voxels/Rock.png")]
    pub rock: Handle<Image>,
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
                .load_collection::<ImageAssets>(),
        )
        .add_systems(OnEnter(GameState::Next), setup)
        .run();
}

fn setup(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let array_texture = create_texture_array(vec![image_assets.rock.clone()], &mut images).unwrap();

    let material = materials.add(VoxelMaterial { array_texture });

    let mut mesh = VoxelMesh::new();

    for x in 0..16 {
        for y in 0..16 {
            for z in 0..16 {
                let position = UVec3::new(x, y, z);
                mesh.add_face(position, VoxelFace::Left, 0);
                mesh.add_face(position, VoxelFace::Right, 0);
                mesh.add_face(position, VoxelFace::Front, 0);
                mesh.add_face(position, VoxelFace::Back, 0);
                mesh.add_face(position, VoxelFace::Top, 0);
                mesh.add_face(position, VoxelFace::Bottom, 0);
            }
        }
    }

    commands.spawn((
        Mesh3d(meshes.add(mesh.build())),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
