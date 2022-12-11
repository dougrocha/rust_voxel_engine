use bevy::{
    core::{Pod, Zeroable},
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::wireframe::WireframePlugin,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{PrimitiveTopology, VertexAttribute},
    },
};

use bevy_inspector_egui::WorldInspectorPlugin;

use rust_game::{
    player::PlayerPlugin,
    world::voxel_data::{FACES, NORMALS, VERTICES},
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
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(WireframePlugin::default())
        // Game State
        .add_state(AppState::InGame)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_light)
        .add_startup_system(load_scene)
        .add_startup_system(load_world)
        .run();
}

/// set up a simple 3D scene
fn setup_light(mut commands: Commands) {
    // light
    commands.insert_resource(AmbientLight {
        color: Color::rgb(0.2, 0.2, 0.2),
        brightness: 0.5,
    });

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

fn load_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 10.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
}

fn load_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = create_block_cube();

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..Default::default()
        }),
        transform: Transform::from_xyz(2.0, 0.2, 0.0),
        ..Default::default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::rgb(0.8, 0.7, 0.6),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, 0.2, 0.0),
        ..Default::default()
    });
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct Vertex {
    position: [f32; 3],
    // color: [f32; 4],
}

fn create_block_cube() -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let positions = cube_positions();
    let colors = cube_colors();
    let normals = cube_normals();

    let mut vertices = Vec::new();
    for i in 0..positions.len() {
        vertices.push(Vertex {
            position: [
                positions[i][0] as f32,
                positions[i][1] as f32,
                positions[i][2] as f32,
            ],
            // color: [
            //     colors[i][0] as f32,
            //     colors[i][1] as f32,
            //     colors[i][2] as f32,
            //     1.0,
            // ],
        });
    }

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(
            vertices
                .iter()
                .map(|v| v.position)
                .collect::<Vec<[f32; 3]>>(),
        ),
    );

    // mesh.insert_attribute(
    //     Mesh::ATTRIBUTE_COLOR,
    //     VertexAttributeValues::Float32x4(
    //         vertices.iter().map(|v| v.color).collect::<Vec<[f32; 4]>>(),
    //     ),
    // );

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(
            normals
                .iter()
                .map(|v| [v[0] as f32, v[1] as f32, v[2] as f32])
                .collect::<Vec<[f32; 3]>>(),
        ),
    );

    mesh.set_indices(Some(Indices::U32(
        (0..positions.len() as u32).collect::<Vec<u32>>(),
    )));

    mesh
}

pub fn cube_positions() -> Vec<[i8; 3]> {
    [
        // front (0, 0, 1)
        [-1, -1, 1],
        [1, -1, 1],
        [-1, 1, 1],
        [-1, 1, 1],
        [1, -1, 1],
        [1, 1, 1],
        // right (1, 0, 0)
        [1, -1, 1],
        [1, -1, -1],
        [1, 1, 1],
        [1, 1, 1],
        [1, -1, -1],
        [1, 1, -1],
        // back (0, 0, -1)
        [1, -1, -1],
        [-1, -1, -1],
        [1, 1, -1],
        [1, 1, -1],
        [-1, -1, -1],
        [-1, 1, -1],
        // left (-1, 0, 0)
        [-1, -1, -1],
        [-1, -1, 1],
        [-1, 1, -1],
        [-1, 1, -1],
        [-1, -1, 1],
        [-1, 1, 1],
        // top (0, 1, 0)
        [-1, 1, 1],
        [1, 1, 1],
        [-1, 1, -1],
        [-1, 1, -1],
        [1, 1, 1],
        [1, 1, -1],
        // bottom (0, -1, 0)
        [-1, -1, -1],
        [1, -1, -1],
        [-1, -1, 1],
        [-1, -1, 1],
        [1, -1, -1],
        [1, -1, 1],
    ]
    .to_vec()
}

pub fn cube_colors() -> Vec<[i8; 3]> {
    [
        // front - blue
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        // right - red
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        // back - yellow
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        [1, 1, 0],
        // left - aqua
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        [0, 1, 1],
        // top - green
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        // bottom - fuchsia
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
        [1, 0, 1],
    ]
    .to_vec()
}

fn cube_normals() -> Vec<[i8; 3]> {
    [
        // front
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        [0, 0, 1],
        // right
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        [1, 0, 0],
        // back
        [0, 0, -1],
        [0, 0, -1],
        [0, 0, -1],
        [0, 0, -1],
        [0, 0, -1],
        [0, 0, -1],
        // left
        [-1, 0, 0],
        [-1, 0, 0],
        [-1, 0, 0],
        [-1, 0, 0],
        [-1, 0, 0],
        [-1, 0, 0],
        // top
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        [0, 1, 0],
        // bottom
        [0, -1, 0],
        [0, -1, 0],
        [0, -1, 0],
        [0, -1, 0],
        [0, -1, 0],
        [0, -1, 0],
    ]
    .to_vec()
}
