mod aabb;
mod block;
mod chunk;
mod game_state;
mod physics;
mod player;
mod texture_array;
mod voxel_material;
mod voxel_mesh;
mod world;

use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;
use chunk::Chunk;
use game_state::GameState;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use voxel_material::VoxelMaterial;
use world::WorldPlugin;

#[derive(Resource)]
struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
}

#[derive(AssetCollection, Resource)]
pub struct ImageAssets {
    #[asset(path = "Voxels/Rock.png")]
    pub rock: Handle<Image>,
}

#[derive(Resource)]
struct VoxelMaterials {
    material: Handle<VoxelMaterial>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
            PlayerPlugin,
            WorldPlugin,
            PhysicsPlugin,
        ))
        .insert_resource(ChunkManager {
            chunks: HashMap::new(),
        })
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<ImageAssets>(),
        )
        .run();
}
