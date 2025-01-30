#![allow(clippy::too_many_arguments)]

mod aabb;
mod block;
mod chunk;
mod game_state;
mod level;
mod loader;
mod physics;
mod player;
mod position;
mod voxel_mesh;

use bevy::prelude::*;
use level::LevelPlugin;
use loader::LoaderPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            LoaderPlugin,
            PlayerPlugin,
            LevelPlugin,
            PhysicsPlugin,
        ))
        .run();
}
