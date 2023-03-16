mod chunk;
mod debug;
mod player;
mod position;

use bevy::prelude::*;
use chunk::ChunkPlugin;
use debug::DebugPlugin;
use player::PlayerPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Engine".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(DebugPlugin)
        // Game State
        .add_plugin(PlayerPlugin)
        .add_plugin(ChunkPlugin)
        .run();
}

// fn setup(
//     mut commands: Commands,
//     mut world_manager: WorldManager,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
// for x in -3..3 {
//     for y in -3..3 {
//         for z in -3..3 {
//             let entity = commands
//                 .spawn(Chunk {
//                     position: IVec3::new(x, y, z),
//                     voxels: VoxelContainer::new(),

//                     entities: Vec::new(),
//                 })
//                 .insert(PbrBundle {
//                     // cube
//                     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//                     material: materials.add(Color::RED.into()),
//                     transform: Transform::from_xyz(
//                         x as f32 * CHUNK_SIZE as f32,
//                         y as f32 * CHUNK_SIZE as f32,
//                         z as f32 * CHUNK_SIZE as f32,
//                     ),
//                     ..Default::default()
//                 })
//                 .id();
//             world_manager.world.add_chunk(IVec3::new(x, y, z), entity);
//         }
//     }
// }
// }
