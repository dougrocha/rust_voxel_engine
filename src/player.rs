use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

#[derive(Component)]
pub struct Player;

impl Player {
    pub fn testingtesting() {}
}

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
            walk_speed: 50.,
            run_speed: 24.,
            friction: 0.8,
            gravity: 9.81,
            jump_height: 1.5,
            max_velocity: 10.,
        }
    }
}

fn setup_player(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 25.0, 0.0),
            ..Default::default()
        },
        Player,
    ));
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut state: ResMut<MouseState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<Player>>,
) {
    let window = window.single();

    let mut delta_state = state.as_mut();
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

fn player_move(
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
