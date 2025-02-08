use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::item::{Item, ItemKind, Material, ToolPart};

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Item>,
    hotbar: [Option<usize>; 9],
    selected: usize,
}

impl Default for Inventory {
    #[allow(clippy::vec_init_then_push)]
    fn default() -> Self {
        let mut items = Vec::new();

        items.push(Item::new(ItemKind::Twig, 1));
        items.push(Item::new(ItemKind::PlantFiber, 1));
        items.push(Item::new(ItemKind::Flint, 1));
        items.push(Item::new(ItemKind::Soil, 1));
        items.push(Item::new(ItemKind::Glass, 1));
        items.push(Item::new(ItemKind::SmallBottle, 1));
        items.push(Item::new(ItemKind::MediumBottle, 1));
        items.push(Item::new(ItemKind::LargeBottle, 1));

        let materials = [
            Material::Twig,
            Material::PlantFiber,
            Material::Flint,
            Material::Glass,
        ];

        for &material in &materials {
            items.push(Item::new(ItemKind::Handle(ToolPart::new(material)), 1));
            items.push(Item::new(ItemKind::Binding(ToolPart::new(material)), 1));
            items.push(Item::new(ItemKind::PickaxeHead(ToolPart::new(material)), 1));

            for &second in &materials {
                for &third in &materials {
                    items.push(Item::new(
                        ItemKind::Pickaxe {
                            handle: ToolPart::new(material),
                            binding: ToolPart::new(second),
                            head: ToolPart::new(third),
                        },
                        1,
                    ));
                }
            }
        }

        Self {
            items,
            hotbar: [None; 9],
            selected: 0,
        }
    }
}

impl Inventory {
    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn select(&mut self, slot: usize) {
        if slot < self.hotbar.len() {
            self.selected = slot;
        }
    }

    pub fn slot(&self) -> usize {
        self.selected
    }

    pub fn hand(&self) -> Option<Item> {
        self.hotbar[self.selected].map(|index| self.items[index])
    }

    pub fn hotbar(&self) -> [Option<&Item>; 9] {
        [
            self.hotbar[0].map(|index| &self.items[index]),
            self.hotbar[1].map(|index| &self.items[index]),
            self.hotbar[2].map(|index| &self.items[index]),
            self.hotbar[3].map(|index| &self.items[index]),
            self.hotbar[4].map(|index| &self.items[index]),
            self.hotbar[5].map(|index| &self.items[index]),
            self.hotbar[6].map(|index| &self.items[index]),
            self.hotbar[7].map(|index| &self.items[index]),
            self.hotbar[8].map(|index| &self.items[index]),
        ]
    }

    pub fn add(&mut self, item: Item) {
        if item.kind.is_stackable() {
            if let Some(existing_item) = self.items.iter_mut().find(|i| i.kind == item.kind) {
                existing_item.count += item.count;
                return;
            }
        }

        self.items.push(item);

        if let Some(slot) = self.hotbar.iter_mut().find(|slot| slot.is_none()) {
            *slot = Some(self.items.len() - 1);
        }
    }

    pub fn set_hotbar(&mut self, slot: usize, item: Option<usize>) {
        if slot < self.hotbar.len() {
            if let Some(index) = item {
                if index >= self.items.len() {
                    return;
                }
            }
            self.hotbar[slot] = item;
        }
    }
}
