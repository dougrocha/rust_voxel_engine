use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_inspector_egui::WorldInspectorPlugin;

use rust_game::{
    player::{MovementSettings, PlayerPlugin},
    world::{
        block::BlockPosition,
        chunk::{
            self,
            chunk_manager::{ChunkConfig, ChunkManager},
            chunks::{ChunkPosition, Direction, CHUNK_SIZE},
        },
        voxel_data::{FACES, FACE_ORDER, VERTICES},
    },
};

const CLEAR_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR_COLOR))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Voxel Engine".into(),
                ..default()
            },
            ..default()
        }))
        .insert_resource(MovementSettings {
            walk_speed: 30.0,
            ..default()
        })
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(WireframePlugin)
        // Game State
        .add_state(AppState::InGame)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_light)
        .add_startup_system(setup_scene)
        .run();
}

/// set up a simple 3D scene
fn setup_light(mut commands: Commands) {
    // light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.2, 0.2, 0.2),
        brightness: 0.5,
    });
    // point light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}

// pub const FACES: [[usize; 4]; 6] = [
//     // top
//     [7, 6, 2, 3],
//     // bottom
//     [0, 1, 5, 4],
//     // front
//     [4, 5, 6, 7],
//     // back
//     [1, 0, 3, 2],
//     // left
//     [0, 4, 7, 3],
//     // right
//     [5, 1, 2, 6],
// ];

// pub const VERTICES: [[f32; 3]; 8] = [
//     [0.0, 0.0, 0.0], // 0
//     [1.0, 0.0, 0.0], // 1
//     [1.0, 1.0, 0.0], // 2
//     [0.0, 1.0, 0.0], // 3
//     [0.0, 0.0, 1.0], // 4
//     [1.0, 0.0, 1.0], // 5
//     [1.0, 1.0, 1.0], // 6
//     [0.0, 1.0, 1.0], // 7
// ];

// pub const FACE_ORDER: [u32; 6] = [0, 1, 2, 0, 2, 3];

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let mut cube = Mesh::new(PrimitiveTopology::TriangleList);

    // let mut vertices: Vec<[f32; 3]> = Vec::new();
    // let mut indices: Vec<u32> = Vec::new();

    // let mut vertex_count = 0;

    // render 3x3x1 plane of cubes
    // for x in 0..16 {
    //     for y in 0..16 {
    //         for z in 0..16 {
    //             for face in FACES {
    //                 for index in face.iter() {
    //                     vertices.push([
    //                         VERTICES[*index][0] + x as f32,
    //                         VERTICES[*index][1] + y as f32,
    //                         VERTICES[*index][2] + z as f32,
    //                     ]);

    //                     for index in 0..6 {
    //                         indices.push(vertex_count + FACE_ORDER[index]);
    //                     }
    //                 }

    //                 // indices.push(vertex_count);
    //                 // indices.push(vertex_count + 1);
    //                 // indices.push(vertex_count + 2);
    //                 // indices.push(vertex_count);
    //                 // indices.push(vertex_count + 2);
    //                 // indices.push(vertex_count + 3);

    //                 vertex_count += 4;
    //             }
    //         }
    //     }
    // }

    // cube.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    // cube.set_indices(Some(Indices::U32(indices)));

    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(cube),
    //         material: materials.add(StandardMaterial {
    //             base_color: Color::RED,
    //             cull_mode: None,
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     },
    //     Wireframe,
    // ));

    // chunk manager
    let mut chunk_manager = ChunkManager::new(ChunkConfig::default());

    let position = ChunkPosition::new(0, 0);
    let position2 = ChunkPosition::new(1, 0);
    let position3 = ChunkPosition::new(2, 0);

    chunk_manager.generate_chunk(position);
    chunk_manager.generate_chunk(position2);
    chunk_manager.generate_chunk(position3);

    let mesh = chunk_manager.get_chunk(position).unwrap().mesh.clone();
    let mesh2 = chunk_manager.get_chunk(position2).unwrap().mesh.clone();
    let mesh3 = chunk_manager.get_chunk(position3).unwrap().mesh.clone();

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..Default::default()
            }),
            ..Default::default()
        },
        Wireframe,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh2),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..Default::default()
            }),
            ..Default::default()
        },
        Wireframe,
    ));

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh3),
            material: materials.add(StandardMaterial {
                base_color: Color::RED,
                ..Default::default()
            }),
            ..Default::default()
        },
        Wireframe,
    ));

    // load a chunk 16 x 16 x 16

    // let mut chunk = Chunk::new(ChunkPosition::new(0, 0, 0));

    // chunk.update_mesh();

    // commands.spawn((
    //     PbrBundle {
    //         mesh: meshes.add(chunk.mesh),
    //         material: materials.add(StandardMaterial {
    //             base_color: Color::RED,
    //             ..Default::default()
    //         }),
    //         ..Default::default()
    //     },
    //     Wireframe,
    // ));
}

// fn load_block_material(
//     asset_server: &Res<AssetServer>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
//     asset_path: &str,
// ) -> Handle<StandardMaterial> {
//     let image_handle = asset_server.load(asset_path);
//     let material_handle = materials.add(StandardMaterial {
//         base_color_texture: Some(image_handle.clone()),
//         alpha_mode: AlphaMode::Opaque,
//         unlit: true,
//         ..default()
//     });
//     material_handle
// }

// fn setup_floor(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let sandstone_material = load_block_material(&asset_server, &mut materials, "sand.png");

//     for x in 0..16 {
//         for z in 0..16 {
//             commands.spawn(PbrBundle {
//                 mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 / 16.0 })),
//                 material: sandstone_material.clone(),
//                 transform: Transform::from_xyz(x as f32 / 16., 0.0, z as f32 / 16.),
//                 ..default()
//             });
//         }
//     }
// }
