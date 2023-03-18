use std::{ops::Mul, thread};

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::AsyncComputeTaskPool,
};
use noise::{MultiFractal, NoiseFn, OpenSimplex, RidgedMulti};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::{player::components::Player, position::world_to_chunk};

use super::{
    components::{AwaitingMesh, BaseChunk, Chunk, DestroyChunk, VoxelContainer},
    mesh::{ao_to_color, generate_mesh, VoxelType},
    resources::{self, ChunkChannel, ChunkQueue, PlayerChunk, World, WorldSeed},
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
    chunks: Query<(&BaseChunk, Entity)>,
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
    destroy_chunk_queue: Query<(&BaseChunk, Entity), With<DestroyChunk>>,
    mut world: ResMut<resources::World>,
) {
    for (chunk, entity) in destroy_chunk_queue.iter() {
        commands.entity(entity).remove::<DestroyChunk>();
        commands.entity(entity).remove::<BaseChunk>();
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

pub fn generate_chunk(chunk_position: IVec3, world_seed: u32) -> VoxelContainer {
    let mut voxels = VoxelContainer::new();

    let ridged_noise: RidgedMulti<OpenSimplex> = RidgedMulti::new(world_seed)
        .set_octaves(3)
        .set_frequency(0.00622);

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let global_x: i32 = chunk_position.x * CHUNK_SIZE as i32 + x as i32;
                let global_y: i32 = chunk_position.y * CHUNK_SIZE as i32 + y as i32;
                let global_z: i32 = chunk_position.z * CHUNK_SIZE as i32 + z as i32;

                let noise_val =
                    ridged_noise.get([global_x as f64, global_y as f64, global_z as f64]) * 45.0;

                let voxel = if global_y as f64 <= noise_val {
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
                    .send(BaseChunk {
                        position: chunk_position,
                        voxels: generate_chunk(chunk_position, world_seed),
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks: Query<(&BaseChunk, Entity), With<AwaitingMesh>>,
) {
    for (chunk, entity) in chunks.iter() {
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

        let aos = ao_to_color(aos);

        mesh.set_indices(Some(Indices::U32(indices)));

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, aos);

        commands
            .entity(entity)
            .insert(PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(
                    chunk.position.as_vec3().mul(CHUNK_SIZE as f32),
                ),
                ..Default::default()
            })
            .remove::<AwaitingMesh>();
    }
}
