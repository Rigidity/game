use bevy::prelude::*;

use crate::{game_state::GameState, player::Player, position::BlockPos};

#[derive(Debug, Clone, Copy)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_ui)
            .add_systems(
                Update,
                (update_position_text, update_fps_text).run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone, Copy, Component)]
struct PositionText;

#[derive(Debug, Clone, Copy, Component)]
struct FpsText;

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        PositionText,
        Text::new("0, 0, 0"),
        Node {
            left: Val::Px(5.0),
            top: Val::Px(5.0),
            ..default()
        },
    ));

    commands.spawn((
        FpsText,
        Text::new("FPS: 0"),
        Node {
            left: Val::Px(5.0),
            top: Val::Px(25.0),
            ..default()
        },
    ));

    commands.spawn((
        ImageNode::new(asset_server.load("Items/Wood.png")),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            width: Val::Px(32.0),
            height: Val::Px(32.0),
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

fn update_fps_text(time: Res<Time>, mut text_query: Query<&mut Text, With<FpsText>>) {
    let fps = 1.0 / time.delta_secs();
    let mut text = text_query.single_mut();
    text.0 = format!("FPS: {:.1}", fps);
}
