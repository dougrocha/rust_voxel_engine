use bevy::{prelude::*, utils::HashMap};

use super::chunk::{chunks::Chunk, ChunkPosition};

#[derive(Default, Resource)]
pub struct ViewDistance {
    pub distance: i32,
}

#[derive(Resource)]
pub struct ChunkMap {
    pub chunks: HashMap<ChunkPosition, Chunk>,
}

impl ChunkMap {
    pub fn new() -> ChunkMap {
        ChunkMap {
            chunks: HashMap::new(),
        }
    }

    pub fn set(&mut self, chunk_position: ChunkPosition, chunk: Chunk) {
        self.chunks.insert(chunk_position, chunk);
    }

    pub fn get(&self, chunk_position: &ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(chunk_position)
    }

    pub fn get_mut(&mut self, chunk_position: &ChunkPosition) -> Option<&mut Chunk> {
        self.chunks.get_mut(chunk_position)
    }

    pub fn remove(&mut self, chunk_position: &ChunkPosition) {
        self.chunks.remove(&chunk_position);
    }
}
