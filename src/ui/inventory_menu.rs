use bevy::{prelude::*, window::PrimaryWindow};
use itertools::Itertools;

use crate::{inventory::Inventory, loader::ItemImages};

use super::{set_grab, ItemImageCache};

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryMenu;

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryItemList;

#[derive(Debug, Clone, Copy, Component)]
pub struct InventoryItem(usize);

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
                BackgroundColor(Color::srgba(0.4, 0.4, 0.4, 0.5)),
                BorderRadius::all(Val::Px(8.0)),
            ))
            .with_child((
                InventoryItemList,
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Column,
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
        Visibility::Inherited
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
        commands.entity(entity).despawn_recursive();
    }

    commands.entity(item_list.single()).with_children(|list| {
        for (index, &item) in inventory.items().iter().enumerate().sorted() {
            let item_texture = item_image_cache.get(item, &mut images, &item_images);

            list.spawn((
                InventoryItem(index),
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(4.0),
                    width: Val::Percent(100.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(4.0)),
                PickingBehavior {
                    should_block_lower: false,
                    is_hoverable: true,
                },
                Interaction::None,
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

                if item.kind.is_stackable() {
                    row.spawn((
                        Text::new(item.count.to_string()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        PickingBehavior::IGNORE,
                    ));
                }
            });
        }
    });
}

pub fn update_item_hover(mut query: Query<(&Interaction, &mut BackgroundColor)>) {
    for (interaction, mut color) in query.iter_mut() {
        color.0 = match interaction {
            Interaction::Hovered => Color::srgb(0.2, 0.2, 0.2),
            Interaction::Pressed => Color::srgb(0.15, 0.15, 0.15),
            _ => Color::NONE,
        };
    }
}

pub fn set_hotbar_selection(
    click: Trigger<Pointer<Click>>,
    mut inventory: ResMut<Inventory>,
    query: Query<&InventoryItem>,
) {
    let Ok(item) = query.get(click.entity()) else {
        return;
    };

    let slot = inventory.slot();
    inventory.set_hotbar(slot, Some(item.0));
}

pub fn clear_hotbar_slot(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if !keys.just_pressed(KeyCode::Backspace) {
        return;
    }

    let slot = inventory.slot();
    inventory.set_hotbar(slot, None);
}
