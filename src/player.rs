mod cursor;
mod interaction;
mod movement;

use bevy::prelude::*;
use cursor::{initial_grab_cursor, player_look, toggle_grab};
use interaction::{break_or_place_block, update_focused_block, FocusedBlock};
use movement::player_move;

use crate::{game_state::GameState, physics::Velocity};

#[derive(Debug, Clone, Copy)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FocusedBlock>()
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_player, initial_grab_cursor),
            )
            .add_systems(
                Update,
                (
                    (player_look, toggle_grab, player_move),
                    update_focused_block,
                    break_or_place_block,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct Player {
    pub on_ground: bool,
}

#[derive(Debug, Clone, Copy, Component)]
struct PlayerCamera;

fn spawn_player(mut commands: Commands) {
    commands
        .spawn((
            Player::default(),
            Transform::from_xyz(10.0, 100.0, 10.0),
            Velocity(Vec3::ZERO),
            Visibility::Inherited,
        ))
        .with_child((
            PlayerCamera,
            Camera3d::default(),
            Transform::from_xyz(0.0, 0.6, 0.0),
            Visibility::Inherited,
        ));
}
