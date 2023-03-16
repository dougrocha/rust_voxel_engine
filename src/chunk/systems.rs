use std::ops::Mul;

use bevy::prelude::*;

use crate::{player::Player, position::world_to_chunk, world_manager::WorldManager};

use super::{
    components::{AwaitingMesh, Chunk, DestroyChunk, VoxelContainer},
    resources::{ChunkQueue, PlayerChunk},
    RenderDistance, World, CHUNK_SIZE,
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
    chunks: Query<(&Chunk, Entity)>,
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
    destroy_chunk_queue: Query<(&Chunk, Entity), With<DestroyChunk>>,
    mut world: ResMut<World>,
) {
    for (chunk, entity) in destroy_chunk_queue.iter() {
        commands.entity(entity).remove::<DestroyChunk>();
        commands.entity(entity).remove::<Chunk>();
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

pub fn generate_chunk(
    mut commands: Commands,
    mut chunk_queue: ResMut<ChunkQueue>,
    world: Res<World>,
) {
    for chunk_position in chunk_queue.generate.drain(..) {
        // do some generation here

        let chunk_entity = world.get_entity(chunk_position).unwrap();

        commands
            .entity(chunk_entity)
            .insert(Chunk {
                position: chunk_position,
                voxels: VoxelContainer::new(),
                entities: Vec::new(),
            })
            .insert(AwaitingMesh);
    }
}

pub fn generate_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks: Query<(&Chunk, Entity), With<AwaitingMesh>>,
) {
    for (chunk, entity) in chunks.iter() {
        commands
            .entity(entity)
            .insert(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_translation(
                    chunk.position.as_vec3().mul(CHUNK_SIZE as f32),
                ),
                ..Default::default()
            })
            .insert(Chunk {
                position: chunk.position,
                voxels: VoxelContainer::new(),
                entities: Vec::new(),
            })
            .remove::<AwaitingMesh>();
    }
}
