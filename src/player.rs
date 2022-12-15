use bevy::{
    ecs::event::ManualEventReader, input::mouse::MouseMotion, prelude::*, window::CursorGrabMode,
};

use crate::{
    camera::FpsCameraComponent,
    window::{cursor_grab, initial_grab_cursor},
};

// Keep track of mouse events, pitch, yaw
#[derive(Resource, Default)]
pub struct MouseState {
    mouse_events: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

#[derive(Resource)]
pub struct MovementSettings {
    pub mouse_sensitivity: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub gravity: f32,
    pub jump_height: f32,
    pub max_velocity: f32,

    pub position: Vec3,
}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseState>()
            .init_resource::<MovementSettings>()
            .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.00012,
            walk_speed: 12.,
            run_speed: 24.,
            friction: 0.8,
            gravity: 9.81,
            jump_height: 1.5,
            max_velocity: 10.,
            position: Vec3::ZERO,
        }
    }
}

fn setup_player(mut commands: Commands, movement_settings: Res<MovementSettings>) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(movement_settings.position),
            ..Default::default()
        },
        FpsCameraComponent,
    ));
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<MouseState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FpsCameraComponent>>,
) {
    if let Some(window) = windows.get_primary() {
        let mut delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in delta_state.mouse_events.iter(&motion) {
                match window.cursor_grab_mode() {
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
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn player_move(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<&mut Transform, With<FpsCameraComponent>>,
) {
    if let Some(window) = windows.get_primary() {
        for mut transform in query.iter_mut() {
            let mut velocity = Vec3::ZERO;
            let local_z = transform.local_z();
            let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
            let right = Vec3::new(local_z.z, 0.0, -local_z.x);

            for key in keyboard_input.get_pressed() {
                match window.cursor_grab_mode() {
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
    } else {
        warn!("No primary window found for `player_move`!");
    }
}
