mod hud;
mod inventory_menu;
mod pause_menu;

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use hud::{
    set_hotbar_slot, spawn_hud, update_fps_text, update_hotbar_display, update_position_text,
};
use inventory_menu::setup_inventory_menu;
use pause_menu::{setup_pause_menu, toggle_pause_menu};

use crate::game_state::{is_unpaused, GameState};

pub use hud::Inventory;

#[derive(Debug, Clone, Copy)]
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Inventory>()
            .add_systems(OnEnter(GameState::Setup), initial_grab_cursor)
            .add_systems(
                OnEnter(GameState::Playing),
                (spawn_hud, setup_pause_menu, setup_inventory_menu),
            )
            .add_systems(
                Update,
                toggle_pause_menu.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    update_position_text,
                    update_fps_text,
                    (update_hotbar_display, set_hotbar_slot).chain(),
                )
                    .run_if(in_state(GameState::Playing).and(is_unpaused)),
            );
    }
}

fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = primary_window.get_single_mut() else {
        warn!("Primary window not found, cursor grab will not be enabled");
        return;
    };
    set_grab(&mut window, true);
}

fn set_grab(window: &mut Window, grab: bool) {
    window.set_cursor_position(Some(window.size() / 2.0));
    window.cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        CursorGrabMode::None
    };
    window.cursor_options.visible = !grab;
}
