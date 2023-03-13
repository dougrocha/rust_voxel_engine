mod mesher;
mod player;

use std::sync::Arc;

use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use player::{Player, PlayerPlugin};

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
        .add_plugin(WorldInspectorPlugin::default())
        // Game State
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_world)
        .init_resource::<World>()
        .insert_resource(RenderDistance {
            horizontal: 4,
            vertical: 2,
        })
        .add_system(poll_chunks_in_view)
        .add_system(poll_chunks_outside_view)
        .run();
}

#[derive(Resource)]
pub struct RenderDistance {
    horizontal: usize,
    vertical: usize,
}

const CHUNK_SIZE: usize = 16;

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<IVec3, Arc<Chunk>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }
}

impl World {
    pub fn chunks_in_render_distance(
        center: &Vec3,
        render_distance: &RenderDistance,
    ) -> Vec<IVec3> {
        let mut chunks = Vec::new();

        let horizontal = render_distance.horizontal as i32;
        let vertical = render_distance.vertical as i32;

        for x in -horizontal..=horizontal {
            for y in -vertical..=vertical {
                for z in -horizontal..=horizontal {
                    let chunk = IVec3::new(
                        (center.x as i32 + x * CHUNK_SIZE as i32) / CHUNK_SIZE as i32,
                        (center.y as i32 + y * CHUNK_SIZE as i32) / CHUNK_SIZE as i32,
                        (center.z as i32 + z * CHUNK_SIZE as i32) / CHUNK_SIZE as i32,
                    );

                    chunks.push(chunk);
                }
            }
        }

        chunks
    }

    pub fn chunks_outside_render_distance(
        &self,
        center: &Vec3,
        render_distance: &RenderDistance,
    ) -> Vec<IVec3> {
        // look through all chunks in the world, and return the positions of the ones that are outside the render distance
        self.chunks
            .iter()
            .filter_map(|(position, _)| {
                let horizontal = render_distance.horizontal as i32;
                let vertical = render_distance.vertical as i32;

                let x = (center.x as i32 - position.x * CHUNK_SIZE as i32).abs();
                let y = (center.y as i32 - position.y * CHUNK_SIZE as i32).abs();
                let z = (center.z as i32 - position.z * CHUNK_SIZE as i32).abs();

                if x > horizontal * CHUNK_SIZE as i32
                    || y > vertical * CHUNK_SIZE as i32
                    || z > horizontal * CHUNK_SIZE as i32
                {
                    Some(*position)
                } else {
                    None
                }
            })
            .collect()
    }
}

pub struct Chunk {
    pub voxels: [Voxel; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
}

pub enum Visibility {
    Empty,
    Transparent,
    Opaque,
}

#[derive(Clone, Copy)]
pub enum Voxel {
    Empty,
    Opaque,
    Transparent,
}

impl Default for Voxel {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Component)]
pub struct NeedsMeshing;

pub fn poll_chunks_in_view(
    player: Query<&Transform, With<Player>>,
    mut world: ResMut<World>,
    render_distance: Res<RenderDistance>,
) {
    let player_position = player.single().translation;

    let chunks = World::chunks_in_render_distance(&player_position, &render_distance);

    for chunk_position in chunks {
        if !world.chunks.contains_key(&chunk_position) {
            //add to world
            let chunk = Chunk {
                voxels: [Voxel::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
            };

            println!("Adding chunk at {:?}", chunk_position);

            world.chunks.insert(chunk_position, Arc::new(chunk));
        }
    }
}

pub fn poll_chunks_outside_view(
    player: Query<&Transform, With<Player>>,
    mut world: ResMut<World>,
    render_distance: Res<RenderDistance>,
) {
    let player_position = player.single().translation;

    let chunks = world.chunks_outside_render_distance(&player_position, &render_distance);

    for chunk_position in chunks {
        if world.chunks.contains_key(&chunk_position) {
            //remove from world
            println!("Removing chunk at {:?}", chunk_position);

            world.chunks.remove(&chunk_position);
        }
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let mut chunk = Chunk::default();

    // for x in 0..CHUNK_SIZE {
    //     for y in 0..CHUNK_SIZE {
    //         for z in 0..CHUNK_SIZE {
    //             let voxel = if y == 8 {
    //                 Voxel::Opaque(2)
    //             } else {
    //                 Voxel::Empty
    //             };

    //             let index = Chunk::linearize(x, y, z);

    //             chunk.voxels[index] = voxel;
    //         }
    //     }
    // }

    // let result = generate_mesh(&chunk);

    // let mut positions = Vec::new();
    // let mut indices = Vec::new();
    // let mut normals = Vec::new();
    // let mut uvs = Vec::new();
    // let mut aos = Vec::new();

    // for face in result.iter_with_ao(&chunk) {
    //     positions.extend_from_slice(&face.positions(1.0)); // Voxel size is 1m
    //     indices.extend_from_slice(&face.indices(positions.len() as u32));
    //     normals.extend_from_slice(&face.normals());
    //     uvs.extend_from_slice(&face.uvs(false, true));
    //     aos.extend_from_slice(&face.aos());
    // }

    // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // mesh.set_indices(Some(Indices::U32(indices)));

    // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, ao_to_vec4(&aos));

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(mesh),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    //     ..Default::default()
    // });

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

// fn ao_to_vec4(ao: &[u32]) -> Vec<[f32; 4]> {
//     ao.iter()
//         .map(|val| match val {
//             0 => [0.1, 0.1, 0.1, 1.0],
//             1 => [0.25, 0.25, 0.25, 1.0],
//             2 => [0.5, 0.5, 0.5, 1.0],
//             _ => [1.0, 1.0, 1.0, 1.0],
//         })
//         .collect()
// }
