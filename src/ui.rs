mod hud;
mod inventory_menu;
mod pause_menu;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::focus::HoverMap,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use hud::{
    set_hotbar_slot, spawn_hud, update_fps_text, update_hotbar_display, update_position_text,
};
use inventory_menu::{setup_inventory_menu, toggle_inventory_menu, update_inventory_menu};
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
                    toggle_inventory_menu.after(toggle_pause_menu),
                    update_inventory_menu,
                )
                    .run_if(in_state(GameState::Playing).and(is_unpaused)),
            )
            .add_systems(Update, update_scroll_position);
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
    window.cursor_options.grab_mode = if grab {
        CursorGrabMode::Confined
    } else {
        window.set_cursor_position(Some(window.size() / 2.0));
        CursorGrabMode::None
    };
    window.cursor_options.visible = !grab;
}

fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 21.0, mouse_wheel_event.y * 21.0),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_x -= dx;
                    scroll_position.offset_y -= dy;
                }
            }
        }
    }
}
