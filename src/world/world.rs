use bevy::{prelude::*, utils::HashMap};

use super::chunk::chunks::{Chunk, ChunkPosition};

pub const DRAW_DISTANCE: i32 = 1;

pub struct ChunkWorld {
    pub chunks: HashMap<ChunkPosition, Chunk>,
}

impl ChunkWorld {
    pub fn new() -> ChunkWorld {
        ChunkWorld {
            chunks: HashMap::new(),
        }
    }

    pub fn render(
        &mut self,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for x in -DRAW_DISTANCE..DRAW_DISTANCE {
            for z in -DRAW_DISTANCE..DRAW_DISTANCE {
                let position = ChunkPosition::new(x, z);

                if !self.chunks.contains_key(&position) {
                    let mut chunk = Chunk::new(position);

                    chunk.generate_blocks();
                    chunk.render(&mut commands, &mut materials, &mut meshes);

                    self.chunks.insert(position, chunk);
                }
            }
        }
    }
}
