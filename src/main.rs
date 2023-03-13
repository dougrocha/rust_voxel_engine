mod mesher;
mod player;

use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    utils::hashbrown::HashMap,
};
use mesher::{generate_mesh, Chunk, Voxel};
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
        // Game State
        .add_plugin(PlayerPlugin)
        .add_plugin(WorldGeneratorPlugin)
        .run();
}

struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_world);
    }
}

const CHUNK_SIZE: usize = 16;

#[derive(Resource)]
struct World {
    chunks: HashMap<IVec3, Arc<Chunk>>,
}

#[derive(Component)]
struct NeedsMeshUpdate;

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    test: Query,
) {
    let mut chunk = Chunk::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel = if y == 8 {
                    Voxel::Opaque(2)
                } else {
                    Voxel::Empty
                };

                let index = Chunk::linearize(x, y, z);

                chunk.voxels[index] = voxel;
            }
        }
    }

    let result = generate_mesh(&chunk);

    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut aos = Vec::new();

    for face in result.iter_with_ao(&chunk) {
        positions.extend_from_slice(&face.positions(1.0)); // Voxel size is 1m
        indices.extend_from_slice(&face.indices(positions.len() as u32));
        normals.extend_from_slice(&face.normals());
        uvs.extend_from_slice(&face.uvs(false, true));
        aos.extend_from_slice(&face.aos());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    mesh.set_indices(Some(Indices::U32(indices)));

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, ao_to_vec4(&aos));

    commands.spawn(PbrBundle {
        mesh: meshes.add(mesh),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ..Default::default()
    });

    // ambient light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(50.0, 50.0, 50.0),
        point_light: PointLight {
            intensity: 600000.,
            range: 100.,
            ..default()
        },
        ..default()
    });
}

fn ao_to_vec4(ao: &[u32]) -> Vec<[f32; 4]> {
    ao.iter()
        .map(|val| match val {
            0 => [0.1, 0.1, 0.1, 1.0],
            1 => [0.25, 0.25, 0.25, 1.0],
            2 => [0.5, 0.5, 0.5, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        })
        .collect()
}
