use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::item::Item;

#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize)]
pub struct Inventory {
    items: Vec<Item>,
    hotbar: [Option<usize>; 9],
    selected: usize,
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
}
