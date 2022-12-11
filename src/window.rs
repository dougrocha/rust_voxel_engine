use bevy::{
    prelude::*,
    window::{CursorGrabMode, Window, Windows},
};

pub fn toggle_grab_cursor(window: &mut Window) {
    match window.cursor_grab_mode() {
        CursorGrabMode::None => {
            window.set_cursor_grab_mode(CursorGrabMode::Confined);
            window.set_cursor_visibility(false);
        }
        _ => {
            window.set_cursor_grab_mode(CursorGrabMode::None);
            window.set_cursor_visibility(true);

            // reset cursor to middle of screen
            window.set_cursor_position(Vec2 {
                x: (window.width() / 2.0),
                y: (window.height() / 2.0),
            });
        }
    }
}

pub fn initial_grab_cursor(mut window: ResMut<Windows>) {
    if let Some(window) = window.get_primary_mut() {
        window.set_cursor_grab_mode(CursorGrabMode::Confined);
        window.set_cursor_visibility(false);
    } else {
        warn!("No primary window found for `initial_grab_cursor`!");
    }
}

pub fn cursor_grab(keyboard_input: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    } else {
        warn!("No primary window found for `cursor_grab`!");
    }
}
