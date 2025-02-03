#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]

mod aabb;
mod block;
mod chunk;
mod game_state;
mod item;
mod level;
mod loader;
mod physics;
mod player;
mod position;
mod ui;
mod voxel_mesh;

use bevy::prelude::*;
use bevy_tokio_tasks::TokioTasksPlugin;
use level::LevelPlugin;
use loader::LoaderPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            TokioTasksPlugin::default(),
            LoaderPlugin,
            PlayerPlugin,
            LevelPlugin,
            PhysicsPlugin,
            UiPlugin,
        ))
        .insert_resource(ClearColor(Color::linear_rgb(0.3, 0.6, 0.9)))
        .run();
}
