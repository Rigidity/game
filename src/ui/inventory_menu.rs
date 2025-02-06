use bevy::{prelude::*, window::PrimaryWindow};
use itertools::Itertools;

use crate::loader::ItemImages;

use super::{set_grab, Inventory, ItemImageCache};

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryMenu;

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryItemList;

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryItem;

pub fn setup_inventory_menu(mut commands: Commands) {
    commands
        .spawn((
            InventoryMenu,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|menu| {
            menu.spawn((
                Node {
                    padding: UiRect::all(Val::Px(16.0)),
                    width: Val::Px(300.0),
                    height: Val::Px(500.0),
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_child((
                InventoryItemList,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    overflow: Overflow::scroll_y(),
                    width: Val::Percent(100.0),
                    height: Val::Auto,
                    margin: UiRect::all(Val::Px(0.0)),
                    ..default()
                },
            ));
        });
}

pub fn toggle_inventory_menu(
    keys: Res<ButtonInput<KeyCode>>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    mut inventory_menu: Query<&mut Visibility, With<InventoryMenu>>,
    mut inventory_item_list: Query<&mut ScrollPosition, With<InventoryItemList>>,
) {
    let opening = keys.just_pressed(KeyCode::KeyE) && inventory_menu.single() == Visibility::Hidden;
    let closing = (keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyE))
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

    inventory_item_list.single_mut().offset_y = 0.0;
}

pub fn update_inventory_menu(
    mut commands: Commands,
    inventory: Res<Inventory>,
    item_images: Res<ItemImages>,
    mut item_image_cache: ResMut<ItemImageCache>,
    mut images: ResMut<Assets<Image>>,
    items: Query<Entity, With<InventoryItem>>,
    item_list: Query<Entity, With<InventoryItemList>>,
) {
    if !inventory.is_changed() {
        return;
    }

    for entity in items.iter() {
        commands.entity(entity).despawn();
    }

    commands.entity(item_list.single()).with_children(|list| {
        for &item in inventory.items().sorted() {
            let item_texture = item_image_cache.get(item, &mut images, &item_images);

            list.spawn((
                InventoryItem,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                PickingBehavior {
                    should_block_lower: false,
                    is_hoverable: true,
                },
            ))
            .with_children(|row| {
                row.spawn((
                    ImageNode::new(item_texture),
                    Node {
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ));

                row.spawn((
                    Text::new(inventory.count(&item).to_string()),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    PickingBehavior::IGNORE,
                ));
            });
        }
    });
}
