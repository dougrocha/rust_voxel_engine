pub mod components;
mod mesh;
mod resources;
mod systems;
mod world_manager;

use bevy::prelude::*;
use tokio::sync::mpsc::channel;

use self::resources::{
    ChunkChannel, ChunkQueue, MeshChannel, PlayerChunk, RenderDistance, World, WorldSeed,
};
use self::systems::{
    build_chunk_mesh, chunk_generation_poll, destroy_chunk_poll, destroy_chunks,
    handle_chunk_generation, handle_chunk_mesh, should_load_chunks, update_player_chunk,
};

pub const CHUNK_SIZE: usize = 32;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .init_resource::<ChunkQueue>()
            .init_resource::<PlayerChunk>()
            .init_resource::<WorldSeed>()
            .insert_resource(RenderDistance {
                horizontal: 12,
                vertical: 6,
            })
            .insert_resource(ChunkChannel(channel(512)))
            .insert_resource(MeshChannel(channel(512)))
            .add_systems((
                update_player_chunk,
                chunk_generation_poll,
                destroy_chunk_poll.run_if(should_load_chunks),
                handle_chunk_generation,
            ))
            .add_system(handle_chunk_mesh)
            .add_system(build_chunk_mesh.after(destroy_chunk_poll))
            .add_system(destroy_chunks);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deref, DerefMut)]
pub struct ChunkPosition(pub IVec3);

impl ChunkPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        ChunkPosition(IVec3::new(x, y, z))
    }

    pub fn to_world(&self) -> IVec3 {
        IVec3::new(
            self.0.x * CHUNK_SIZE as i32,
            self.0.y * CHUNK_SIZE as i32,
            self.0.z * CHUNK_SIZE as i32,
        )
    }

    pub fn from_world(world_position: &IVec3) -> Self {
        ChunkPosition::new(
            world_position.x / CHUNK_SIZE as i32,
            world_position.y / CHUNK_SIZE as i32,
            world_position.z / CHUNK_SIZE as i32,
        )
    }

    pub fn from_player(player_position: &Vec3) -> Self {
        ChunkPosition::new(
            player_position.x.floor() as i32 / CHUNK_SIZE as i32,
            player_position.y.floor() as i32 / CHUNK_SIZE as i32,
            player_position.z.floor() as i32 / CHUNK_SIZE as i32,
        )
    }

    pub fn neighbors(&self) -> Vec<ChunkPosition> {
        let mut neighbors = Vec::with_capacity(26);

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }

                    neighbors.push(ChunkPosition::new(self.0.x + x, self.0.y + y, self.0.z + z));
                }
            }
        }

        neighbors
    }
}
