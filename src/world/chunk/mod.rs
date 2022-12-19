use bevy::{
    prelude::*,
    utils::{hashbrown, HashMap},
};
use noise::NoiseFn;

use self::chunks::Chunk;

use super::{
    block::{Block, BlockPosition, BlockType},
    world::ChunkMap,
};

pub mod chunks;
pub mod mesh;

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

    pub fn from_block_position(block_position: &BlockPosition) -> ChunkPosition {
        ChunkPosition {
            x: (block_position.x / CHUNK_SIZE) as i32,
            z: (block_position.z / CHUNK_SIZE) as i32,
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

    pub fn set_block(&mut self, block_position: &BlockPosition, block: Block) {
        self.blocks[block_position.x as usize][block_position.y as usize]
            [block_position.z as usize] = block;
    }

    pub fn get_block(&self, block_position: &BlockPosition) -> &Block {
        &self.blocks[block_position.x as usize][block_position.y as usize]
            [block_position.z as usize]
    }

    pub fn get_block_mut(&mut self, block_position: &BlockPosition) -> &mut Block {
        &mut self.blocks[block_position.x as usize][block_position.y as usize]
            [block_position.z as usize]
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

pub fn generate_height_map(chunk_position: &ChunkPosition, seed: u32, scale: f64) -> Vec<f64> {
    let noise = noise::Perlin::new(seed);

    let mut height_map: Vec<f64> = Vec::with_capacity((CHUNK_SIZE * CHUNK_SIZE) as usize);

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let x = x as f64 + (chunk_position.x * CHUNK_SIZE) as f64;
            let z = z as f64 + (chunk_position.z * CHUNK_SIZE) as f64;

            let height = noise.get([x / scale, z / scale]) * 32.0 + 64.0;

            height_map.push(height);
        }
    }

    height_map
}

pub fn generate_terrain(chunk_position: &ChunkPosition, seed: u32, scale: f64) -> ChunkArray {
    let mut chunk_array = ChunkArray::new();

    let height_map = generate_height_map(chunk_position, seed, scale);

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let x = x as f64 + (chunk_position.x * CHUNK_SIZE) as f64;
            let z = z as f64 + (chunk_position.z * CHUNK_SIZE) as f64;

            for y in 0..height_map[(x as usize) + (z as usize) * CHUNK_SIZE as usize] as i32 {
                chunk_array.set_block(
                    &BlockPosition::new(x as i32, y, z as i32),
                    Block::new(BlockType::GRASS),
                )
            }
        }
    }

    chunk_array
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
