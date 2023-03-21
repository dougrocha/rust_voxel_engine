use std::ops::Mul;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::AsyncComputeTaskPool,
    utils::FloatOrd,
};
use bracket_noise::prelude::*;
use itertools::Itertools;

use crate::{player::components::Player, position::world_to_chunk};

use super::{
    components::{AwaitingMesh, BaseChunk, Chunk, ChunkBundle, ChunkWithNeighbors, DestroyChunk},
    mesh::{ao_to_color, generate_mesh, Voxel, VoxelType},
    resources::{
        self, ChunkChannel, ChunkQueue, MeshChannel, MeshedChunk, PlayerChunk, World, WorldSeed,
    },
    world_manager::WorldManager,
    RenderDistance, CHUNK_SIZE,
};

pub fn should_load_chunks(player_chunk: Res<PlayerChunk>) -> bool {
    player_chunk.is_changed()
}

pub fn update_player_chunk(
    mut player_chunk: ResMut<PlayerChunk>,
    player: Query<&Transform, With<Player>>,
) {
    let player_transform = player.single().translation;

    let new_chunk_position = world_to_chunk(&player_transform);

    if new_chunk_position != player_chunk.0 {
        player_chunk.set(new_chunk_position);
    }
}

pub fn destroy_chunk_poll(
    mut commands: Commands,
    chunks: Query<(&ChunkBundle, Entity)>,
    render_distance: Res<RenderDistance>,
    player_chunk: Res<PlayerChunk>,
) {
    for (chunk, entity) in chunks.iter() {
        if World::can_render(&player_chunk.0, &chunk.position, &render_distance) {
            continue;
        } else {
            commands.entity(entity).insert(DestroyChunk);
        }
    }
}

pub fn destroy_chunks(
    mut commands: Commands,
    destroy_chunk_queue: Query<(&ChunkBundle, Entity), With<DestroyChunk>>,
    mut world: ResMut<resources::World>,
) {
    for (chunk, entity) in destroy_chunk_queue.iter() {
        commands.entity(entity).remove::<DestroyChunk>();
        commands.entity(entity).remove::<ChunkBundle>();
        world.remove_chunk(&chunk.position);
        commands.entity(entity).despawn_recursive();
    }
}

pub fn chunk_generation_poll(
    player: Query<&Transform, With<Player>>,
    mut commands: Commands,
    mut world_manager: WorldManager,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    let position = player.single().translation;
    let center_chunk = world_to_chunk(&position);

    for chunk_position in world_manager.chunks_positions_in_render_distance(center_chunk) {
        if world_manager.world.get_entity(chunk_position).is_none() {
            let entity = commands.spawn_empty().id();

            world_manager.world.add_chunk(chunk_position, entity);

            chunk_queue.generate.push(chunk_position);
        }
    }
}

pub fn generate_chunk(chunk_position: IVec3, world_seed: u64) -> BaseChunk {
    let mut voxels = BaseChunk::new();

    let mut noise = FastNoise::seeded(world_seed);
    noise.set_noise_type(NoiseType::Simplex);
    noise.set_fractal_type(FractalType::RigidMulti);
    noise.set_fractal_octaves(4);
    noise.set_fractal_gain(0.6);
    noise.set_fractal_lacunarity(2.0);
    noise.set_frequency(1.0);

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let global_x: i32 = chunk_position.x * CHUNK_SIZE as i32 + x as i32;
                let global_y: i32 = chunk_position.y * CHUNK_SIZE as i32 + y as i32;
                let global_z: i32 = chunk_position.z * CHUNK_SIZE as i32 + z as i32;

                let noise_val = noise.get_noise3d(
                    global_x as f32 / 100.,
                    global_y as f32 / 100.,
                    global_z as f32 / 100.,
                ) * 45.0;

                let voxel = if global_y as f32 <= noise_val {
                    VoxelType::Opaque(1)
                } else {
                    VoxelType::Empty
                };

                voxels.0[BaseChunk::linearize(UVec3::new(x as u32, y as u32, z as u32))] = voxel;
            }
        }
    }

    voxels
}

pub fn handle_chunk_generation(
    mut commands: Commands,
    mut chunk_queue: ResMut<ChunkQueue>,
    world: Res<World>,
    world_seed: Res<WorldSeed>,
    mut chunk_channel: ResMut<ChunkChannel>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let world_seed = world_seed.0.clone();

    for chunk_position in chunk_queue.generate.drain(..) {
        let sender = chunk_channel.0 .0.clone();

        thread_pool
            .spawn(async move {
                sender
                    .send(ChunkBundle {
                        position: chunk_position,
                        data: generate_chunk(chunk_position, world_seed),
                        entities: Vec::new(),
                    })
                    .await
                    .ok();
            })
            .detach();
    }

    chunk_queue.generate.clear();

    while let Ok(chunk) = chunk_channel.0 .1.try_recv() {
        let chunk_entity = world.get_entity(chunk.position).unwrap();

        commands
            .entity(chunk_entity)
            .insert(chunk)
            .insert(AwaitingMesh);
    }
}

pub fn handle_chunk_mesh(
    mut commands: Commands,
    chunks: Query<&ChunkBundle, With<AwaitingMesh>>,
    mut chunk_queue: ResMut<ChunkQueue>,
    world_manager: WorldManager,
    player_chunk: Res<PlayerChunk>,
) {
    for (count, chunk) in chunks
        .iter()
        .sorted_unstable_by_key(|key| {
            FloatOrd(key.position.as_vec3().distance(player_chunk.0.as_vec3()))
        })
        .enumerate()
    {
        if count > 5 {
            return;
        }

        if world_manager.world.check_neighbors(chunk.position) {
            if let Some(neighbors) = world_manager.neighboring_chunks(chunk.position) {
                if let Ok(neighbors) = neighbors.try_into() {
                    chunk_queue.await_mesh.push((
                        chunk.position,
                        chunk.data.clone(),
                        Box::new(neighbors),
                    ));

                    if let Some(chunk_entity) = world_manager.world.get_entity(chunk.position) {
                        commands.entity(chunk_entity).remove::<AwaitingMesh>();
                    }
                }
            }
        }
    }
}

pub fn build_chunk_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_queue: ResMut<ChunkQueue>,
    mut mesh_channel: ResMut<MeshChannel>,
    world: Res<World>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (chunk_position, chunk, neighbors) in chunk_queue.await_mesh.drain(..) {
        let sender = mesh_channel.0 .0.clone();

        thread_pool
            .spawn(async move {
                let chunk_with_neighbors = ChunkWithNeighbors::new(&chunk, &neighbors);

                sender
                    .send(generate_final_mesh(&chunk_with_neighbors, chunk_position))
                    .await
                    .ok();
            })
            .detach();
    }

    chunk_queue.await_mesh.clear();

    while let Ok(meshed_chunk) = mesh_channel.0 .1.try_recv() {
        if let Some(chunk_entity) = world.get_entity(meshed_chunk.position) {
            commands.entity(chunk_entity).insert(PbrBundle {
                mesh: meshes.add(meshed_chunk.mesh),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(
                    meshed_chunk.position.as_vec3().mul(CHUNK_SIZE as f32),
                ),
                ..Default::default()
            });
        }
    }
}

pub fn generate_final_mesh<C, T>(chunk: &C, chunk_position: IVec3) -> MeshedChunk
where
    C: Chunk<Output = T>,
    T: Voxel,
{
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut aos = Vec::new();

    let result = generate_mesh(chunk);

    for face in result.iter_with_ao(chunk) {
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, ao_to_color(aos));

    MeshedChunk {
        mesh,
        position: chunk_position,
    }
}
