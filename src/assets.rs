use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

#[derive(AssetCollection, Resource)]
pub struct VoxelAssets {
    #[asset(path = "Voxels/Rock.png")]
    pub rock: Handle<Image>,
}

impl VoxelAssets {
    pub fn handles(&self) -> Vec<Handle<Image>> {
        vec![self.rock.clone()]
    }

    pub fn rock() -> u32 {
        0
    }
}
