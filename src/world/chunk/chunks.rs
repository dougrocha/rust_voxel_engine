use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::NoiseFn;

use crate::world::{
    block::{Block, BlockPosition, BlockType},
    voxel_data::{FACES, FACE_ORDER, NORMALS, UVS, VERTICES},
};

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

#[derive(Debug)]
pub struct Chunk {
    pub blocks: ChunkArray,
    pub position: ChunkPosition,

    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,

    noise: noise::SuperSimplex,

    vertex_count: u32,

    pub mesh: Mesh,
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Chunk {
        let blocks = ChunkArray::new();

        Chunk {
            blocks,
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            noise: noise::SuperSimplex::new(123),
            vertex_count: 0,
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
        }
    }

    pub fn generate_blocks(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = self.noise.get([
                    (self.position.x * CHUNK_SIZE + x) as f64 / 16.0,
                    (self.position.z * CHUNK_SIZE + z) as f64 / 16.0,
                ]) * 16.0
                    + 64.0;

                for y in 0..CHUNK_HEIGHT {
                    let block_position = BlockPosition::new(x, y, z);

                    if (y as f64) < height {
                        self.set_block(block_position, Block::new(BlockType::DIRT));
                    } else {
                        self.set_block(block_position, Block::new(BlockType::AIR));
                    }
                }
            }
        }
    }

    pub fn generate_mesh(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block_position = BlockPosition::new(x, y, z);

                    if self.get_block(block_position).unwrap().block_type == BlockType::AIR {
                        continue;
                    }

                    for direction in 0..6 {
                        let direction = match direction {
                            0 => Direction::UP,
                            1 => Direction::DOWN,
                            2 => Direction::LEFT,
                            3 => Direction::RIGHT,
                            4 => Direction::FRONT,
                            5 => Direction::BACK,
                            _ => panic!("Invalid direction"),
                        };

                        let position_in_direction =
                            self.get_position_in_direction(block_position, direction);

                        if self.check_block(position_in_direction) {
                            self.create_face(block_position, direction);
                        }
                    }
                }
            }
        }
    }

    pub fn apply_mesh(&mut self) {
        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices.clone());
        // self.mesh
        // .insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone());

        self.mesh
            .set_indices(Some(Indices::U32(self.indices.clone())));
    }

    pub fn create_face(&mut self, position: BlockPosition, direction: Direction) {
        let position = BlockPosition {
            x: position.x + self.position.x * CHUNK_SIZE,
            y: position.y,
            z: position.z + self.position.z * CHUNK_SIZE,
        };

        let vertices = self.get_face_vertices(position, direction);

        vertices.iter().for_each(|vertex| {
            self.vertices.push(*vertex);
        });

        // indices

        for index in 0..6 {
            self.indices.push(self.vertex_count + FACE_ORDER[index]);
        }

        self.vertex_count += 4;
    }

    pub fn get_normal(&self, direction: Direction) -> [f32; 3] {
        NORMALS[direction as usize]
    }

    pub fn get_uvs(&self, direction: Direction) -> Vec<[f32; 2]> {
        let mut uvs: Vec<[f32; 2]> = Vec::new();

        uvs
    }

    pub fn get_face_vertices(
        &self,
        position: BlockPosition,
        direction: Direction,
    ) -> Vec<[f32; 3]> {
        let mut vertices: Vec<[f32; 3]> = Vec::new();

        for index in FACES[direction as usize] {
            let vertex = [
                position.x as f32 + VERTICES[index][0],
                position.y as f32 + VERTICES[index][1],
                position.z as f32 + VERTICES[index][2],
            ];

            vertices.push(vertex);
        }

        vertices
    }

    pub fn check_block(&self, position: BlockPosition) -> bool {
        if position.x < 0
            || position.x >= CHUNK_SIZE
            || position.y < 0
            || position.y >= CHUNK_HEIGHT
            || position.z < 0
            || position.z >= CHUNK_SIZE
        {
            return true;
        }

        return self.get_block(position).unwrap().is_transparent();
    }

    pub fn get_position_in_direction(
        &self,
        position: BlockPosition,
        direction: Direction,
    ) -> BlockPosition {
        match direction {
            Direction::UP => BlockPosition::new(position.x, position.y + 1, position.z),
            Direction::DOWN => BlockPosition::new(position.x, position.y - 1, position.z),
            Direction::LEFT => BlockPosition::new(position.x - 1, position.y, position.z),
            Direction::RIGHT => BlockPosition::new(position.x + 1, position.y, position.z),
            Direction::FRONT => BlockPosition::new(position.x, position.y, position.z + 1),
            Direction::BACK => BlockPosition::new(position.x, position.y, position.z - 1),
        }
    }

    pub fn get_block(&self, position: BlockPosition) -> Option<&Block> {
        Some(&self.blocks.blocks[position.x as usize][position.y as usize][position.z as usize])
    }

    pub fn get_block_mut(&mut self, position: BlockPosition) -> Option<&mut Block> {
        Some(&mut self.blocks.blocks[position.x as usize][position.y as usize][position.z as usize])
    }

    pub fn set_block(&mut self, position: BlockPosition, block: Block) {
        self.blocks.blocks[position.x as usize][position.y as usize][position.z as usize] = block;
    }

    pub fn get_block_type(&self, position: BlockPosition) -> Option<BlockType> {
        Some(self.get_block(position)?.block_type.clone())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    UP,
    DOWN,
    FRONT,
    BACK,
    LEFT,
    RIGHT,
}
