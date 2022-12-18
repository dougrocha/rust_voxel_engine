use bevy::{
    prelude::*,
    utils::{hashbrown, HashMap},
};
use noise::NoiseFn;

use self::chunks::Chunk;

use super::{
    block::{Block, BlockType},
    world::ChunkMap,
};

pub mod chunks;

pub const CHUNK_SIZE: i32 = 16;
pub const CHUNK_HEIGHT: i32 = 256;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPosition {
    pub x: i32,
    pub z: i32,
}

impl ChunkPosition {
    pub fn new(x: i32, z: i32) -> ChunkPosition {
        ChunkPosition { x, z }
    }

    pub fn from_world_position(world_position: Vec3) -> ChunkPosition {
        ChunkPosition {
            x: (world_position.x / CHUNK_SIZE as f32).floor() as i32,
            z: (world_position.z / CHUNK_SIZE as f32).floor() as i32,
        }
    }

    pub fn distance_to(&self, other: &ChunkPosition) -> f32 {
        (((self.x - other.x).pow(2) + (self.z - other.z).pow(2)) as f32).sqrt()
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

#[derive(Default, Resource)]
pub struct ChunkQueue {
    pub create: Vec<ChunkPosition>,
    pub remove: Vec<ChunkPosition>,
}

pub fn create_chunk(
    mut chunk_queue: ResMut<ChunkQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut commands: Commands,
) {
    chunk_queue.create.drain(..).for_each(|chunk_position| {
        chunk_entities.insert(
            &chunk_position,
            commands.spawn(Chunk::new(chunk_position)).id(),
        );
    })
}

pub fn destroy_chunks(
    mut chunk_queue: ResMut<ChunkQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
    mut chunks: ResMut<ChunkMap>,
    mut commands: Commands,
) {
    chunk_queue.remove.drain(..).for_each(|chunk_position| {
        commands
            .entity(chunk_entities.detach_entity(&chunk_position).unwrap())
            .despawn();
        chunks.remove(&chunk_position);
    })
}

pub fn generate_height_map(seed: u32, chunk_position: &ChunkPosition) -> ChunkArray {
    let mut chunk = ChunkArray::new();

    let noise = noise::OpenSimplex::new(seed);

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let height = (noise.get([
                (chunk_position.x * CHUNK_SIZE + x) as f64 / 16.0,
                (chunk_position.z * CHUNK_SIZE + z) as f64 / 16.0,
            ]) * 16.0) as i32;

            for y in 0..CHUNK_HEIGHT {
                if y < height {
                    chunk.blocks[x as usize][y as usize][z as usize] = Block::new(BlockType::STONE);
                } else if y == height {
                    chunk.blocks[x as usize][y as usize][z as usize] = Block::new(BlockType::GRASS);
                } else {
                    chunk.blocks[x as usize][y as usize][z as usize] = Block::new(BlockType::AIR);
                }
            }
        }
    }

    chunk
}

#[derive(Default, Resource)]
pub struct ChunkEntities(HashMap<ChunkPosition, Entity>);

impl ChunkEntities {
    pub fn get(&self, chunk_position: &ChunkPosition) -> Option<&Entity> {
        self.0.get(&chunk_position)
    }

    pub fn insert(&mut self, chunk_position: &ChunkPosition, entity: Entity) {
        self.0.insert(*chunk_position, entity);
    }

    pub fn contains_key(&self, chunk_position: &ChunkPosition) -> bool {
        self.0.contains_key(&chunk_position)
    }

    pub fn detach_entity(&mut self, chunk_position: &ChunkPosition) -> Option<Entity> {
        self.0.remove(&chunk_position)
    }

    pub fn remove(&mut self, chunk_position: &ChunkPosition) {
        self.0.remove(&chunk_position);
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> hashbrown::hash_map::Iter<ChunkPosition, Entity> {
        self.0.iter()
    }

    pub fn iter_keys(&self) -> hashbrown::hash_map::Keys<ChunkPosition, Entity> {
        self.0.keys()
    }
}
