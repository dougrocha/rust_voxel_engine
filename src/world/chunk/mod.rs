use super::block::{Block, BlockType};

pub mod chunks;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 48;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> ChunkPosition {
        ChunkPosition { x, z }
    }
}

#[derive(Debug)]
pub struct ChunkArray {
    pub blocks: [[[Block; CHUNK_SIZE as usize]; CHUNK_HEIGHT as usize]; CHUNK_SIZE as usize],
}

impl ChunkArray {
    pub fn new() -> ChunkArray {
        let blocks = [[[Block::new(BlockType::DEFAULT); CHUNK_SIZE as usize];
            CHUNK_HEIGHT as usize]; CHUNK_SIZE as usize];

        ChunkArray { blocks }
    }
}

#[derive(Clone, Copy)]
pub struct FMask {
    pub block_type: BlockType,
    pub normal: i8,
}

impl FMask {
    pub fn new(block_type: BlockType, normal: i8) -> FMask {
        FMask { block_type, normal }
    }
}
