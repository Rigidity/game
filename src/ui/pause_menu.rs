use bevy::{prelude::*, window::PrimaryWindow};

use crate::game_state::Paused;

use super::{hud::Hud, inventory_menu::InventoryMenu, set_grab};

#[derive(Debug, Clone, Copy, Component)]
#[require(Node(pause_menu_node), BackgroundColor(pause_menu_bg))]
pub struct PauseMenu;

fn pause_menu_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

fn pause_menu_bg() -> BackgroundColor {
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
}

pub fn setup_pause_menu(mut commands: Commands) {
    commands
        .spawn((PauseMenu, Visibility::Hidden))
        .with_children(|_menu| {});
}

pub fn toggle_pause_menu(
    mut paused: ResMut<Paused>,
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut pause_menu: Query<&mut Visibility, (With<PauseMenu>, Without<Hud>)>,
    mut hud: Query<&mut Visibility, (With<Hud>, Without<PauseMenu>)>,
    inventory_menu: Query<&Visibility, (With<InventoryMenu>, Without<PauseMenu>, Without<Hud>)>,
) {
    if !keys.just_pressed(KeyCode::Escape) || inventory_menu.single() != Visibility::Hidden {
        return;
    }

    let mut window = primary_window.single_mut();

    let should_grab = paused.0;
    set_grab(&mut window, should_grab);

    let mut pause_menu = pause_menu.single_mut();
    *pause_menu = if should_grab {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };

    let mut hud = hud.single_mut();
    *hud = if should_grab {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };

    paused.0 = !paused.0;
}
