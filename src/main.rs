use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::{Wireframe, WireframePlugin},
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};

use bevy_inspector_egui::WorldInspectorPlugin;

use rust_game::{
    player::{MovementSettings, PlayerPlugin},
    world::chunk::chunks::{Chunk, ChunkPosition},
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
            position: Vec3::new(0.0, 80.0, 0.0),
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
        .add_startup_system(setup_world)
        .run();
}

/// set up a simple 3D scene
fn setup_light(mut commands: Commands) {
    //sky light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.80,
    });
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut chunk = Chunk::new(ChunkPosition::new(0, 0));

    chunk.generate_blocks();

    chunk.render(&mut commands, &mut materials, &mut meshes);
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
