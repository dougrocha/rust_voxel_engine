use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};

use super::chunk::{chunks::Chunk, ChunkPosition};

#[derive(Default, Resource)]
pub struct ViewDistance {
    pub distance: i32,
}

#[derive(Resource)]
pub struct ChunkMap {
    pub chunks: HashMap<ChunkPosition, Arc<Chunk>>,
}

#[derive(Resource)]
pub struct DirtyChunks(pub Vec<ChunkPosition>);

impl ChunkMap {
    pub fn new() -> ChunkMap {
        ChunkMap {
            chunks: HashMap::new(),
        }
    }

    pub fn set(&mut self, chunk_position: ChunkPosition, chunk: Chunk) {
        self.chunks.insert(chunk_position, Arc::new(chunk));
    }

    pub fn get(&self, chunk_position: &ChunkPosition) -> Option<Arc<Chunk>> {
        self.chunks
            .get(chunk_position)
            .map(|chunk| Arc::clone(chunk))
    }

    pub fn get_mut(&mut self, chunk_position: &ChunkPosition) -> Option<&mut Chunk> {
        self.chunks
            .get_mut(chunk_position)
            .map(|chunk| Arc::get_mut(chunk).unwrap())
    }

    pub fn remove(&mut self, chunk_position: &ChunkPosition) {
        self.chunks.remove(chunk_position);
    }
}
