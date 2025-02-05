use bevy::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
#[require(Node(inventory_menu_node), BackgroundColor(inventory_menu_bg))]
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

fn inventory_menu_bg() -> BackgroundColor {
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5))
}

pub fn setup_inventory_menu(mut commands: Commands) {
    commands
        .spawn((InventoryMenu, Visibility::Hidden))
        .with_children(|_menu| {});
}
