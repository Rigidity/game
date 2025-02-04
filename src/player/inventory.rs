use bevy::{prelude::*, utils::HashMap};

use crate::item::Item;

#[derive(Debug, Default, Clone, Resource)]
pub struct Inventory {
    items: HashMap<Item, (usize, Handle<Image>)>,
    hotbar: [Option<Item>; 9],
    selected_slot: usize,
}

impl Inventory {
    pub fn add(&mut self, item: Item, count: usize, asset_server: &AssetServer) {
        self.items
            .entry(item)
            .or_insert((0, asset_server.load(item.get_texture_path())))
            .0 += count;

        if self.hotbar.contains(&Some(item)) {
            return;
        }

        if let Some(slot) = self.hotbar.iter().position(|item| item.is_none()) {
            self.hotbar[slot] = Some(item);
        }
    }

    pub fn get_hotbar_slot(&self, slot: usize) -> Option<Item> {
        self.hotbar.get(slot).copied().flatten()
    }

    pub fn get_item_count(&self, item: &Item) -> usize {
        self.items.get(item).map_or(0, |(count, _)| *count)
    }

    pub fn get_item_texture(&self, item: &Item) -> Handle<Image> {
        self.items
            .get(item)
            .map_or(Handle::default(), |(_, texture)| texture.clone())
    }
}

// Component to mark hotbar slot entities
#[derive(Component)]
pub struct HotbarSlot(usize);

// Component to mark item display entities
#[derive(Component)]
pub struct ItemDisplay;

pub fn spawn_inventory(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn((Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            right: Val::Px(20.0),
            bottom: Val::Px(20.0),
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(2.0),
            ..default()
        },))
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
}

pub fn update_hotbar_display(
    mut commands: Commands,
    inventory: Res<Inventory>,
    asset_server: Res<AssetServer>,
    mut hotbar_slots: Query<(Entity, &mut ImageNode, &HotbarSlot)>,
    item_displays: Query<Entity, With<ItemDisplay>>,
) {
    // Remove existing item displays
    for entity in item_displays.iter() {
        commands.entity(entity).despawn();
    }

    // Update each slot with current inventory items
    for (slot_entity, mut image_node, hotbar_slot) in hotbar_slots.iter_mut() {
        if let Some(item) = inventory.get_hotbar_slot(hotbar_slot.0) {
            let item_count = inventory.get_item_count(&item);
            let item_texture = inventory.get_item_texture(&item);

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
                parent.spawn((
                    Text::new(item_count.to_string()),
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
            });
        }

        if hotbar_slot.0 == inventory.selected_slot {
            image_node.image = asset_server.load("Slots/Selected.png");
        } else {
            image_node.image = asset_server.load("Slots/Hotbar.png");
        }
    }
}

pub fn set_hotbar_slot(keys: Res<ButtonInput<KeyCode>>, mut inventory: ResMut<Inventory>) {
    if keys.just_pressed(KeyCode::Digit1) {
        inventory.selected_slot = 0;
    } else if keys.just_pressed(KeyCode::Digit2) {
        inventory.selected_slot = 1;
    } else if keys.just_pressed(KeyCode::Digit3) {
        inventory.selected_slot = 2;
    } else if keys.just_pressed(KeyCode::Digit4) {
        inventory.selected_slot = 3;
    } else if keys.just_pressed(KeyCode::Digit5) {
        inventory.selected_slot = 4;
    } else if keys.just_pressed(KeyCode::Digit6) {
        inventory.selected_slot = 5;
    } else if keys.just_pressed(KeyCode::Digit7) {
        inventory.selected_slot = 6;
    } else if keys.just_pressed(KeyCode::Digit8) {
        inventory.selected_slot = 7;
    } else if keys.just_pressed(KeyCode::Digit9) {
        inventory.selected_slot = 8;
    }
}
