pub mod movement {
    use bevy::{
        ecs::system::Res,
        prelude::*,
        window::{CursorGrabMode, PrimaryWindow},
    };

    use crate::player::{components::Player, resources::MovementSettings};

    pub fn player_move(
        keyboard_input: Res<Input<KeyCode>>,
        time: Res<Time>,
        window: Query<&Window, With<PrimaryWindow>>,
        settings: Res<MovementSettings>,
        mut query: Query<&mut Transform, With<Player>>,
    ) {
        let window = window.single();

        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
            let right = Vec3::new(local_z.z, 0.0, -local_z.x);

            for key in keyboard_input.get_pressed() {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => match key {
                        KeyCode::W => velocity += forward,
                        KeyCode::S => velocity -= forward,
                        KeyCode::A => velocity -= right,
                        KeyCode::D => velocity += right,
                        KeyCode::Space => velocity += Vec3::Y,
                        KeyCode::LShift => velocity -= Vec3::Y,
                        _ => (),
                    },
                }
            }

            velocity = velocity.normalize_or_zero();

            transform.translation += velocity * time.delta_seconds() * settings.walk_speed;
        }
    }
}

pub mod camera {
    use crate::player::{
        components::Player,
        resources::{MouseState, MovementSettings},
    };
    use bevy::{
        input::mouse::MouseMotion,
        prelude::*,
        window::{CursorGrabMode, PrimaryWindow},
    };

    /// Handles looking around if cursor is locked
    pub fn player_look(
        settings: Res<MovementSettings>,
        window: Query<&Window, With<PrimaryWindow>>,
        mut delta_state: ResMut<MouseState>,
        motion: Res<Events<MouseMotion>>,
        mut query: Query<&mut Transform, With<Player>>,
    ) {
        let window = window.single();

        let delta_state = delta_state.as_mut();

        for mut transform in query.iter_mut() {
            for ev in delta_state.mouse_events.iter(&motion) {
                match window.cursor.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        delta_state.pitch -=
                            (settings.mouse_sensitivity * ev.delta.y * window_scale).to_radians();
                        delta_state.yaw -=
                            (settings.mouse_sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
                    * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    }

    pub fn toggle_grab_cursor(window: &mut Window) {
        match window.cursor.grab_mode {
            CursorGrabMode::None => {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }
            _ => {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;

                let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);

                // reset cursor to middle of screen
                window.set_cursor_position(Some(center));
            }
        }
    }

    pub fn initial_grab_cursor(mut window: Query<&mut Window, With<PrimaryWindow>>) {
        let _window = window.single_mut();

        // window.cursor.grab_mode = CursorGrabMode::Confined;
        // window.cursor.visible = false;
    }

    pub fn cursor_grab(
        keyboard_input: Res<Input<KeyCode>>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
    ) {
        let mut window = window.single_mut();

        if keyboard_input.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(&mut window);
        }
    }
}
