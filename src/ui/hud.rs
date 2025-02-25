use bevy::prelude::*;

use crate::{inventory::Inventory, loader::ItemImages, player::Player, position::BlockPos};

use super::ItemImageCache;

#[derive(Debug, Clone, Copy, Component)]
pub struct Hud;

#[derive(Debug, Clone, Copy, Component)]
pub struct PositionText;

#[derive(Debug, Clone, Copy, Component)]
pub struct FpsText;

#[derive(Debug, Clone, Copy, Component)]
pub struct HotbarSlot(usize);

#[derive(Debug, Clone, Copy, Component)]
pub struct ItemDisplay;

pub fn spawn_hud(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((
            Hud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(0.0),
                left: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            PickingBehavior::IGNORE,
        ))
        .with_children(|hud| {
            hud.spawn((
                PositionText,
                Text::new("0, 0, 0"),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(5.0),
                    top: Val::Px(5.0),
                    ..default()
                },
            ));

            hud.spawn((
                FpsText,
                Text::new("FPS: 0"),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(5.0),
                    top: Val::Px(25.0),
                    ..default()
                },
            ));

            hud.spawn((
                Text::new("+"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    margin: UiRect::all(Val::Auto),
                    ..default()
                },
                PickingBehavior::IGNORE,
            ));

            hud.spawn(Node {
                position_type: PositionType::Absolute,
                left: Val::Px(20.0),
                right: Val::Px(20.0),
                bottom: Val::Px(20.0),
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|root| {
                for i in 0..9 {
                    root.spawn((
                        ImageNode::new(asset_server.load(if i == 0 {
                            "Slots/Selected.png"
                        } else {
                            "Slots/Hotbar.png"
                        })),
                        Node {
                            width: Val::Px(48.0),
                            height: Val::Px(48.0),
                            ..default()
                        },
                        HotbarSlot(i),
                    ));
                }
            });
        });
}

pub fn update_position_text(
    player_query: Query<&Transform, With<Player>>,
    mut text_query: Query<&mut Text, With<PositionText>>,
) {
    let player_transform = player_query.single();
    let pos = BlockPos::from_world(player_transform.translation);

    let mut text = text_query.single_mut();
    text.0 = format!("{}, {}, {}", pos.x, pos.y, pos.z);
}

pub fn update_fps_text(time: Res<Time>, mut text_query: Query<&mut Text, With<FpsText>>) {
    let fps = 1.0 / time.delta_secs();
    let mut text = text_query.single_mut();
    text.0 = format!("FPS: {:.1}", fps);
}

pub fn update_hotbar_display(
    mut commands: Commands,
    inventory: Res<Inventory>,
    asset_server: Res<AssetServer>,
    item_images: Res<ItemImages>,
    mut item_image_cache: ResMut<ItemImageCache>,
    mut images: ResMut<Assets<Image>>,
    mut hotbar_slots: Query<(Entity, &mut ImageNode, &HotbarSlot)>,
    item_displays: Query<Entity, With<ItemDisplay>>,
) {
    if !inventory.is_changed() {
        return;
    }

    // Remove existing item displays
    for entity in item_displays.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Update each slot with current inventory items
    let hotbar = inventory.hotbar();

    for (slot_entity, mut image_node, hotbar_slot) in hotbar_slots.iter_mut() {
        if let Some(item) = hotbar[hotbar_slot.0] {
            let item_texture = item_image_cache.get(*item, &mut images, &item_images);

            // Spawn the item image and count inside the slot
            commands.entity(slot_entity).with_children(|parent| {
                // Spawn item image
                parent.spawn((
                    ImageNode::new(item_texture),
                    Node {
                        top: Val::Px(8.0),
                        left: Val::Px(8.0),
                        width: Val::Px(32.0),
                        height: Val::Px(32.0),
                        ..default()
                    },
                    ItemDisplay,
                ));

                // Spawn item count text
                if item.kind.is_stackable() {
                    parent.spawn((
                        Text::new(item.count.to_string()),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(4.0),
                            right: Val::Px(4.0),
                            ..default()
                        },
                        ItemDisplay,
                    ));
                }
            });
        }

        if hotbar_slot.0 == inventory.slot() {
            image_node.image = asset_server.load("Slots/Selected.png");
        } else {
            image_node.image = asset_server.load("Slots/Hotbar.png");
        }
    }
}

pub fn set_hotbar_slot(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if keys.just_pressed(KeyCode::Digit1) {
        inventory.select(0);
    } else if keys.just_pressed(KeyCode::Digit2) {
        inventory.select(1);
    } else if keys.just_pressed(KeyCode::Digit3) {
        inventory.select(2);
    } else if keys.just_pressed(KeyCode::Digit4) {
        inventory.select(3);
    } else if keys.just_pressed(KeyCode::Digit5) {
        inventory.select(4);
    } else if keys.just_pressed(KeyCode::Digit6) {
        inventory.select(5);
    } else if keys.just_pressed(KeyCode::Digit7) {
        inventory.select(6);
    } else if keys.just_pressed(KeyCode::Digit8) {
        inventory.select(7);
    } else if keys.just_pressed(KeyCode::Digit9) {
        inventory.select(8);
    }
}
