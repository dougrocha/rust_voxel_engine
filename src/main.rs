pub mod camera;
pub mod chunk;
pub mod debug;
pub mod terrain;
pub mod voxel;
pub mod world;

use std::time::Instant;

use bevy::prelude::*;

use crate::{
    camera::PlayerCameraPlugin, chunk::ChunkData, debug::DebugPlugin, voxel::Voxel,
    world::WorldPlugin,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins,))
        .add_plugins(PlayerCameraPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(DebugPlugin)
        .add_systems(Startup, setup_environment)
        .add_systems(Startup, spawn_large_chunks)
        .run();
}

fn setup_environment(mut commands: Commands) {
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 50.0, 4.0),
    ));
}

fn spawn_large_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut world: ResMut<world::WorldManager>,
) {
    let start_time = Instant::now();

    // let seed = 12345;
    // let size = 10;
    // world.generate_world(size, seed);
    //
    // for x in 0..size {
    //     for z in 0..size {
    //         let chunk_pos = IVec3::new(x, 0, z);
    //
    //         if let Some(chunk) = world.get_chunk(&chunk_pos) {
    //             let mesh = chunk.generate_mesh(&world);
    //
    //             commands.spawn((
    //                 Mesh3d(meshes.add(mesh)),
    //                 MeshMaterial3d(materials.add(StandardMaterial {
    //                     base_color: Color::WHITE,
    //                     unlit: false, // Enable lighting for better visuals
    //                     ..default()
    //                 })),
    //                 chunk.get_world_transform(),
    //             ));
    //         }
    //     }
    // }
    //
    // dbg!(start_time.elapsed() * 1000);
}
