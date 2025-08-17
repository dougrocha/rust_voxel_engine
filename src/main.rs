pub mod camera;
pub mod voxel;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};

use crate::{
    camera::PlayerCameraPlugin,
    voxel::{Chunk, Voxel},
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
        ))
        .add_plugins(PlayerCameraPlugin)
        .add_systems(Startup, setup_environment)
        .add_systems(Startup, spawn_test_chunk)
        .run();
}

fn setup_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}

fn spawn_test_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Create a new chunk at origin
    let mut chunk = Chunk::new(IVec3::ZERO);

    // Add some test voxels to create a small structure
    // Bottom layer (y=0)
    chunk.set_voxel(Voxel::Dirt, 0, 0, 0);
    chunk.set_voxel(Voxel::Dirt, 1, 0, 0);
    chunk.set_voxel(Voxel::Dirt, 0, 0, 1);
    chunk.set_voxel(Voxel::Dirt, 1, 0, 1);

    chunk.set_voxel(Voxel::Dirt, 4, 0, 0);

    // Generate mesh from voxel data
    let mesh = chunk.generate_mesh();

    // Spawn the chunk as a renderable entity
    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))), // Brownish color
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
