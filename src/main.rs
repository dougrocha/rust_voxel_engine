use bevy::prelude::*;

use bevy_voxel_game::{camera::CameraPlugin, debug::DebugPlugin, world::WorldPlugin};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .add_plugins(DebugPlugin)
        .add_plugins((CameraPlugin, WorldPlugin))
        .run();
}
