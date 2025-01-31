use bevy::prelude::*;

use crate::{game_state::GameState, player::Player, position::BlockPos};

#[derive(Debug, Clone, Copy)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_ui)
            .add_systems(
                Update,
                update_position_text.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct PositionText;

fn setup_ui(mut commands: Commands) {
    commands.spawn((
        PositionText,
        Text::new("0, 0, 0"),
        Node {
            left: Val::Px(5.0),
            top: Val::Px(5.0),
            ..default()
        },
    ));
}

fn update_position_text(
    player_query: Query<&Transform, With<Player>>,
    mut text_query: Query<&mut Text, With<PositionText>>,
) {
    let player_transform = player_query.single();
    let pos = BlockPos::from_world(player_transform.translation);

    let mut text = text_query.single_mut();
    text.0 = format!("{}, {}, {}", pos.x, pos.y, pos.z);
}
