use bevy::{input::mouse::MouseMotion, prelude::*, window::CursorGrabMode};

#[derive(Component)]
pub struct PlayerCamera {
    pub accel: f32,
    pub max_speed: f32,
    pub sensitivity: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        PlayerCamera {
            accel: 1.5,
            max_speed: 0.5,
            sensitivity: 5.0,
            friction: 1.0,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn movement_axis(input: &Res<ButtonInput<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}

fn forward_vector(rotation: &Quat) -> Vec3 {
    rotation.mul_vec3(Vec3::Z).normalize()
}

fn forward_walk_vector(rotation: &Quat) -> Vec3 {
    let f = forward_vector(rotation);
    let f_flattened = Vec3::new(f.x, 0.0, f.z).normalize();
    if f_flattened.is_nan() {
        Vec3::new(0.0, 0.0, 1.0) // if we're looking directly down, assume forward is up Z axis
    } else {
        f_flattened
    }
}

fn strafe_vector(rotation: &Quat) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Quat::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
}

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut PlayerCamera, &mut Transform)>,
) {
    for (mut options, mut transform) in query.iter_mut() {
        let (axis_h, axis_v, axis_float) = {
            (
                movement_axis(&keyboard_input, KeyCode::KeyD, KeyCode::KeyA),
                movement_axis(&keyboard_input, KeyCode::KeyS, KeyCode::KeyW),
                movement_axis(&keyboard_input, KeyCode::Space, KeyCode::ShiftLeft),
            )
        };

        let rotation = transform.rotation;
        let accel: Vec3 = (strafe_vector(&rotation) * axis_h)
            + (forward_walk_vector(&rotation) * axis_v)
            + (Vec3::Y * axis_float);
        let accel: Vec3 = if accel.length() != 0.0 {
            accel.normalize() * options.accel
        } else {
            Vec3::ZERO
        };

        let friction: Vec3 = if options.velocity.length() != 0.0 {
            options.velocity.normalize() * -1.0 * options.friction
        } else {
            Vec3::ZERO
        };

        options.velocity += accel * time.delta_secs();

        // clamp within max speed
        if options.velocity.length() > options.max_speed {
            options.velocity = options.velocity.normalize() * options.max_speed;
        }

        let delta_friction = friction * time.delta_secs();

        options.velocity =
            if (options.velocity + delta_friction).signum() != options.velocity.signum() {
                Vec3::ZERO
            } else {
                options.velocity + delta_friction
            };

        transform.translation += options.velocity;
    }
}

fn mouse_motion_system(
    time: Res<Time>,
    mouse_lock_state: Res<MouseLockState>,
    mut mouse_motion_event_reader: EventReader<MouseMotion>,
    mut query: Query<(&mut PlayerCamera, &mut Transform)>,
) {
    if !mouse_lock_state.is_locked {
        return;
    }

    let mut delta: Vec2 = Vec2::ZERO;
    for event in mouse_motion_event_reader.read() {
        delta += event.delta;
    }
    if delta.is_nan() {
        return;
    }

    for (mut options, mut transform) in query.iter_mut() {
        options.yaw -= delta.x * options.sensitivity * time.delta_secs();
        options.pitch += delta.y * options.sensitivity * time.delta_secs();

        options.pitch = options.pitch.clamp(-89.9, 89.9);
        // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

        let yaw_radians = options.yaw.to_radians();
        let pitch_radians = options.pitch.to_radians();

        transform.rotation = Quat::from_axis_angle(Vec3::Y, yaw_radians)
            * Quat::from_axis_angle(-Vec3::X, pitch_radians);
    }
}

#[derive(Resource, Default)]
struct MouseLockState {
    is_locked: bool,
}

fn cursor_lock_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut mouse_lock_state: ResMut<MouseLockState>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Escape) {
        mouse_lock_state.is_locked = !mouse_lock_state.is_locked;

        if mouse_lock_state.is_locked {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(Camera3d::default())
        .insert(PlayerCamera::default())
        .insert(Transform::from_xyz(80.0, 50.0, 90.0));
}

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MouseLockState>()
            .add_systems(Startup, setup_camera)
            .add_systems(Update, cursor_lock_system)
            .add_systems(Update, camera_movement_system)
            .add_systems(Update, mouse_motion_system);
    }
}
