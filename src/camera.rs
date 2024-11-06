use crate::{Player, VIEW_MODEL_RENDER_LAYER};
use bevy::{
    input::mouse::MouseMotion, prelude::*, render::view::RenderLayers, window::CursorGrabMode,
};
use std::f32::consts::FRAC_PI_2;

#[derive(Component)]
pub struct WorldCamera;

#[derive(Resource)]
pub struct PlayerSettings {
    pub mouse_sensitivity: f32,
    pub walk_speed: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 0.002,
            walk_speed: 25.0,
        }
    }
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerSettings>()
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                (
                    handle_keyboard_inputs,
                    handle_mouse_inputs,
                    handle_cursor_grab,
                ),
            );
    }
}

fn setup_camera(mut commands: Commands) {
    commands
        .spawn((
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // INFO: https://bevyengine.org/examples/camera/first-person-view-model/
            parent.spawn((
                WorldCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));

            // Spawn view model camera.
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        // Bump the order to render on top of the world model.
                        order: 1,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));
        });
}

fn handle_keyboard_inputs(
    mut query: Query<&mut Transform, With<Player>>,
    window: Query<&Window>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    settings: Res<PlayerSettings>,
) {
    let window = window.single();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let mut transform = query.single_mut();

    let mut velocity = Vec3::ZERO;
    let local_z = transform.local_z();
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
    let right = Vec3::new(local_z.z, 0.0, -local_z.x);

    for key in keyboard_input.get_pressed() {
        match key {
            KeyCode::KeyW => velocity += forward,
            KeyCode::KeyS => velocity -= forward,
            KeyCode::KeyA => velocity -= right,
            KeyCode::KeyD => velocity += right,
            _ => {}
        }
    }

    velocity = velocity.normalize_or_zero();

    transform.translation += velocity * time.delta_seconds() * settings.walk_speed;
}

fn handle_mouse_inputs(
    mut player: Query<&mut Transform, With<Player>>,
    window: Query<&Window>,
    mut mouse_motion: EventReader<MouseMotion>,
    settings: Res<PlayerSettings>,
) {
    let window = window.single();
    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let mut transform = player.single_mut();

    for motion in mouse_motion.read() {
        if motion.delta == Vec2::ZERO {
            break;
        }

        let delta_yaw = -motion.delta.x * settings.mouse_sensitivity;
        let delta_pitch = -motion.delta.y * settings.mouse_sensitivity;

        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let yaw = yaw + delta_yaw;

        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

fn handle_cursor_grab(keyboard_input: Res<ButtonInput<KeyCode>>, mut window: Query<&mut Window>) {
    let mut window = window.single_mut();

    if keyboard_input.just_pressed(KeyCode::Escape) {
        match window.cursor.grab_mode {
            bevy::window::CursorGrabMode::None => {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }
            _ => {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;

                let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
                window.set_cursor_position(Some(center));
            }
        }
    }
}
