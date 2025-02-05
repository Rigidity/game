use bevy::{prelude::*, window::PrimaryWindow};

use super::{hud::Hud, set_grab};

#[derive(Debug, Clone, Copy, Component)]
#[require(Node(inventory_menu_node))]
pub struct InventoryMenu;

fn inventory_menu_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(0.0),
        left: Val::Px(0.0),
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

pub fn setup_inventory_menu(mut commands: Commands) {
    commands
        .spawn((InventoryMenu, Visibility::Hidden))
        .with_children(|_menu| {});
}

pub fn toggle_inventory_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut inventory_menu: Query<&mut Visibility, (With<InventoryMenu>, Without<Hud>)>,
    mut hud: Query<&mut Visibility, (With<Hud>, Without<InventoryMenu>)>,
) {
    let opening = keys.just_pressed(KeyCode::KeyI) && inventory_menu.single() == Visibility::Hidden;
    let closing = (keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyI))
        && inventory_menu.single() != Visibility::Hidden;

    if !opening && !closing {
        return;
    }

    let mut window = primary_window.single_mut();

    let should_grab = closing;
    set_grab(&mut window, should_grab);

    let mut inventory_menu = inventory_menu.single_mut();
    *inventory_menu = if should_grab {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };

    let mut hud = hud.single_mut();
    *hud = if should_grab {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
}
