#![allow(clippy::too_many_arguments)]

mod aabb;
mod block;
mod chunk;
mod game_state;
mod physics;
mod player;
mod position;
mod texture_array;
mod voxel_material;
mod voxel_mesh;
mod world;

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use game_state::GameState;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use voxel_material::VoxelMaterial;
use world::{WorldMap, WorldPlugin};

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
        .insert_resource(WorldMap::new(42))
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<ImageAssets>(),
        )
        .run();
}
