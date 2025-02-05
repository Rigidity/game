use bevy::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Setup,
    Playing,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Resource)]
pub struct Paused(pub bool);

pub fn is_unpaused(paused: Res<Paused>) -> bool {
    !paused.0
}
