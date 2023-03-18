pub mod components;
mod mesh;
mod resources;
mod systems;
mod world_manager;

use bevy::prelude::*;
use tokio::sync::mpsc::channel;

use self::resources::{ChunkChannel, ChunkQueue, PlayerChunk, RenderDistance, World, WorldSeed};
use self::systems::{
    chunk_generation_poll, destroy_chunk_poll, destroy_chunks, handle_chunk_generation,
    handle_chunk_mesh, should_load_chunks, update_player_chunk,
};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .init_resource::<ChunkQueue>()
            .init_resource::<PlayerChunk>()
            .init_resource::<WorldSeed>()
            .insert_resource(RenderDistance {
                horizontal: 8,
                vertical: 6,
            })
            .insert_resource(ChunkChannel(channel(512)))
            .add_systems((
                update_player_chunk,
                chunk_generation_poll,
                handle_chunk_generation,
                handle_chunk_mesh,
                destroy_chunk_poll.run_if(should_load_chunks),
                destroy_chunks,
            ));
    }
}
