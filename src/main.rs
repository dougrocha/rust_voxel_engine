mod mesher;
mod player;

use std::{ops::Div, sync::Arc};

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    math::Vec3Swizzles,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use futures_lite::future;
use hashbrown::HashMap;
use mesher::generate_mesh;
use noise::{NoiseFn, Perlin, RidgedMulti};
use player::{Player, PlayerPlugin};
use rayon::prelude::*;

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
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugin(WorldInspectorPlugin::default())
        // Game State
        .add_plugin(PlayerPlugin)
        .init_resource::<World>()
        .insert_resource(RenderDistance {
            horizontal: 6,
            vertical: 4,
        })
        .add_startup_system(setup_world)
        .add_system(poll_chunks_in_view)
        .add_system(load_meshes_for_chunks)
        .add_system(render_chunks)
        .add_system(poll_chunks_outside_view)
        .add_systems(
            (
                poll_chunks_in_view,
                poll_chunks_outside_view,
                load_meshes_for_chunks,
                render_chunks,
            )
                .chain(),
        )
        .run();
}

pub fn setup_world(mut commands: Commands) {
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

#[derive(Resource)]
pub struct RenderDistance {
    horizontal: usize,
    vertical: usize,
}

const CHUNK_SIZE: usize = 16;

#[derive(Resource)]
pub struct World {
    pub chunks: HashMap<IVec3, Arc<Box<Chunk>>>,
}

impl Default for World {
    fn default() -> Self {
        Self {
            chunks: HashMap::default(),
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
        let mut vec = Vec::new();

        for (chunk_position, _) in self.chunks.iter() {
            let outside = chunk_position
                .xz()
                .as_vec2()
                .distance(center.xz() / CHUNK_SIZE as f32)
                .abs()
                .floor() as i32
                > render_distance.horizontal as i32
                || (chunk_position.y as i32 - (center.y.div(CHUNK_SIZE as f32)) as i32).abs()
                    > render_distance.vertical as i32;

            if outside {
                vec.push(*chunk_position);
            }
        }

        vec

        // self.chunks
        //     .iter()
        //     .filter_map(|(position, _)| {
        //         let horizontal = render_distance.horizontal as i32;
        //         let vertical = render_distance.vertical as i32;

        //         let x = (center.x as i32 - position.x * CHUNK_SIZE as i32).abs();
        //         let y = (center.y as i32 - position.y * CHUNK_SIZE as i32).abs();
        //         let z = (center.z as i32 - position.z * CHUNK_SIZE as i32).abs();

        //         if x > horizontal * CHUNK_SIZE as i32
        //             || y > vertical * CHUNK_SIZE as i32
        //             || z > horizontal * CHUNK_SIZE as i32
        //         {
        //             Some(position.clone())
        //         } else {
        //             None
        //         }
        //     })
        //     .collect()
    }

    pub fn is_outside_render_distance(
        center: &Vec3,
        chunk_position: &IVec3,
        render_distance: &RenderDistance,
    ) -> bool {
        let horizontal = render_distance.horizontal as i32;
        let vertical = render_distance.vertical as i32;

        let x = (center.x as i32 - chunk_position.x * CHUNK_SIZE as i32).abs();
        let y = (center.y as i32 - chunk_position.y * CHUNK_SIZE as i32).abs();
        let z = (center.z as i32 - chunk_position.z * CHUNK_SIZE as i32).abs();

        x > horizontal * CHUNK_SIZE as i32
            || y > vertical * CHUNK_SIZE as i32
            || z > horizontal * CHUNK_SIZE as i32
    }
}

#[derive(Clone, Copy)]
pub struct Chunk {
    pub voxels: [Voxel; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
    pub entity: Entity,
    pub spawned: bool,
}

impl Chunk {
    fn linearize(x: usize, y: usize, z: usize) -> usize {
        x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE)
    }

    fn delinearize(mut index: usize) -> (usize, usize, usize) {
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        index -= z * (CHUNK_SIZE * CHUNK_SIZE);

        let y = index / CHUNK_SIZE;
        index -= y * CHUNK_SIZE;

        let x = index;

        (x, y, z)
    }

    fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::linearize(x, y, z)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Empty,
    Transparent,
    Opaque,
}

pub const EMPTY: Visibility = Visibility::Empty;
pub const TRANSPARENT: Visibility = Visibility::Transparent;
pub const OPAQUE: Visibility = Visibility::Opaque;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Voxel {
    #[default]
    Empty,
    Opaque,
    Transparent,
}

impl Voxel {
    fn visibility(&self) -> Visibility {
        match self {
            Self::Empty => Visibility::Empty,
            Self::Opaque => Visibility::Opaque,
            Self::Transparent => Visibility::Transparent,
        }
    }
}

#[derive(Component)]
pub struct NeedsMeshing;

pub fn poll_chunks_in_view(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    mut world: ResMut<World>,
    render_distance: Res<RenderDistance>,
) {
    let player_position = player.single().translation;

    let chunks = World::chunks_in_render_distance(&player_position, &render_distance);

    let mut ridged_multi = RidgedMulti::<Perlin>::new(3246725);

    ridged_multi.frequency = 0.3;
    ridged_multi.octaves = 3;

    for chunk_position in chunks {
        if !world.chunks.contains_key(&chunk_position) {
            //add to world
            let id = commands.spawn_empty().id();

            let mut chunk = Chunk {
                voxels: [Voxel::default(); CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
                entity: id,
                spawned: false,
            };

            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        let height = (ridged_multi.get([
                            (chunk_position.x * (CHUNK_SIZE + x) as i32) as f64 / 16.0,
                            (chunk_position.z * (CHUNK_SIZE + z) as i32) as f64 / 16.0,
                        ]) * 8.0) as i32;

                        let voxel = if y as i32 + chunk_position.y * (CHUNK_SIZE as i32) < height {
                            Voxel::Opaque
                        } else {
                            Voxel::Empty
                        };

                        let index = Chunk::linearize(x, y, z);

                        chunk.voxels[index] = voxel;
                    }
                }
            }

            world
                .chunks
                .insert(chunk_position, Arc::new(Box::new(chunk)));
        }
    }
}

pub fn poll_chunks_outside_view(
    mut commands: Commands,
    player: Query<&Transform, With<Player>>,
    world: Res<World>,
    render_distance: Res<RenderDistance>,
) {
    let player_position = player.single().translation;

    for (chunk_position, chunk) in world.chunks.iter() {
        if World::is_outside_render_distance(&player_position, &chunk_position, &render_distance) {
            let chunk = Arc::clone(chunk);

            if let Some(mut entity) = commands.get_entity(chunk.entity) {
                entity.despawn();
            }
        }
    }
}

pub fn render_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshing_tasks: Query<(Entity, &mut MeshingTask)>,
) {
    for (entity, mut task) in &mut meshing_tasks {
        if let Some((mesh, chunk_pos)) = future::block_on(future::poll_once(&mut task.0)) {
            if let Some(_chunk_entity) = commands.get_entity(entity) {
                commands.get_or_spawn(entity).insert(PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                    transform: Transform::from_translation(Vec3::new(
                        chunk_pos.x as f32 * CHUNK_SIZE as f32,
                        chunk_pos.y as f32 * CHUNK_SIZE as f32,
                        chunk_pos.z as f32 * CHUNK_SIZE as f32,
                    )),
                    ..Default::default()
                });

                commands.entity(entity).remove::<MeshingTask>();
            }
        }
    }
}

#[derive(Component)]
pub struct MeshingTask(Task<(Mesh, IVec3)>);

fn load_meshes_for_chunks(mut commands: Commands, mut world: ResMut<World>) {
    let thread_pool = AsyncComputeTaskPool::get();

    // loop through chunks and spawn a single cube per chunk
    for (position, chunk) in world.chunks.iter_mut() {
        let chunk_clone = Arc::clone(chunk);

        if chunk.spawned {
            continue;
        }

        let position = *position;

        let task = thread_pool.spawn(async move {
            let result = generate_mesh(&chunk_clone);

            let mut positions = Vec::new();
            let mut indices = Vec::new();
            let mut normals = Vec::new();
            let mut uvs = Vec::new();
            let mut aos = Vec::new();

            for face in result.iter_with_ao(&chunk_clone) {
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

            (mesh, position)
        });

        if let Some(mut chunk_entity) = commands.get_entity(chunk.entity) {
            chunk_entity.insert(MeshingTask(task));
        }

        let mut chunk = Arc::make_mut(chunk);
        chunk.spawned = true;
    }
}

fn ao_to_vec4(ao: &[u32]) -> Vec<[f32; 4]> {
    ao.par_iter()
        .map(|val| match val {
            0 => [0.1, 0.1, 0.1, 1.0],
            1 => [0.25, 0.25, 0.25, 1.0],
            2 => [0.5, 0.5, 0.5, 1.0],
            _ => [1.0, 1.0, 1.0, 1.0],
        })
        .collect()
}
