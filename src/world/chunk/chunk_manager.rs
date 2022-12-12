use bevy::utils::hashbrown::HashMap;

use super::chunks::{Chunk, ChunkPosition, CHUNK_HEIGHT, CHUNK_SIZE};

pub struct ChunkConfig {
    pub chunk_size: i32,
    pub chunk_height: i32,
}

impl Default for ChunkConfig {
    fn default() -> Self {
        Self {
            chunk_size: CHUNK_SIZE,
            chunk_height: CHUNK_HEIGHT,
        }
    }
}

pub struct ChunkManager {
    pub chunks: HashMap<ChunkPosition, Chunk>,
    pub chunk_config: ChunkConfig,
}

impl ChunkManager {
    pub fn new(chunk_config: ChunkConfig) -> ChunkManager {
        let chunks: HashMap<ChunkPosition, Chunk> = HashMap::new();

        ChunkManager {
            chunks,
            chunk_config,
        }
    }

    pub fn get_chunk(&self, position: ChunkPosition) -> Option<&Chunk> {
        self.chunks.get(&position)
    }

    pub fn get_chunk_mut(&mut self, position: ChunkPosition) -> Option<&mut Chunk> {
        self.chunks.get_mut(&position)
    }

    pub fn generate_chunk(&mut self, position: ChunkPosition) {
        let mut chunk = Chunk::new(position);

        chunk.generate_blocks();

        chunk.generate_mesh();

        chunk.apply_mesh();

        self.chunks.insert(position, chunk);
    }
}
