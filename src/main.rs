pub mod camera;
pub mod chunk;
pub mod voxel;
pub mod world;

use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    prelude::*,
    text::FontSmoothing,
};

use crate::{camera::PlayerCameraPlugin, chunk::Chunk, voxel::Voxel, world::WorldPlugin};

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
        .add_plugins(WorldPlugin)
        .add_systems(Startup, setup_environment)
        .add_systems(Startup, spawn_chunk_meshes)
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
        Transform::from_xyz(4.0, 50.0, 4.0),
    ));
}

fn spawn_chunk_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<world::World>,
) {
    let mut chunk1 = Chunk::new(IVec3::new(0, 0, 0));
    chunk1.set_voxel(Voxel::Stone, 0, 0, 0);
    chunk1.set_voxel(Voxel::Stone, 1, 0, 0);
    chunk1.set_voxel(Voxel::Stone, 31, 0, 0);

    let mut chunk2 = Chunk::new(IVec3::new(1, 0, 0));
    chunk2.set_voxel(Voxel::Grass, 0, 0, 0);

    world.add_chunk(IVec3::new(0, 0, 0), chunk1);
    world.add_chunk(IVec3::new(1, 0, 0), chunk2);

    if let Some(chunk) = world.get_chunk(&IVec3::new(0, 0, 0)) {
        let (mesh, _stats) = chunk.generate_mesh_with_stats(&world);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            chunk.get_world_transform(),
        ));
    }

    if let Some(chunk) = world.get_chunk(&IVec3::new(1, 0, 0)) {
        let (mesh, _stats) = chunk.generate_mesh_with_stats(&world);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            chunk.get_world_transform(),
        ));
    }
}
