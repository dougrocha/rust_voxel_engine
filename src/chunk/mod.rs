pub mod components;
mod mesh;
mod resources;
mod systems;
mod world_manager;

use bevy::prelude::*;

use self::resources::{ChunkQueue, PlayerChunk, RenderDistance, World, WorldSeed};
use self::systems::{
    chunk_generation_poll, destroy_chunk_poll, destroy_chunks, generate_chunk, handle_chunk_mesh,
    should_load_chunks, update_player_chunk,
};

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize =
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .init_resource::<ChunkQueue>()
            .init_resource::<PlayerChunk>()
            .init_resource::<WorldSeed>()
            .insert_resource(RenderDistance {
                horizontal: 3,
                vertical: 3,
            })
            .add_systems((
                update_player_chunk,
                chunk_generation_poll,
                generate_chunk,
                handle_chunk_mesh,
                destroy_chunk_poll.run_if(should_load_chunks),
                destroy_chunks,
            ));
    }
}
