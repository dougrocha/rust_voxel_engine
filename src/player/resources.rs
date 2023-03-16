use bevy::{ecs::event::ManualEventReader, input::mouse::MouseMotion, prelude::*};

// Keep track of mouse events, pitch, yaw
#[derive(Resource, Default)]
pub struct MouseState {
    pub mouse_events: ManualEventReader<MouseMotion>,
    pub pitch: f32,
    pub yaw: f32,
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
