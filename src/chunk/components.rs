use bevy::prelude::{Component, Entity, IVec3, UVec3};

use super::{mesh::VoxelType, ChunkPosition, CHUNK_SIZE, CHUNK_VOLUME};

#[derive(Component)]
pub struct AwaitingMesh;

#[derive(Component)]
pub struct DestroyChunk;

pub trait Chunk {
    type Output;

    const X: usize;
    const Y: usize;
    const Z: usize;

    fn size() -> usize {
        CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE
    }

    fn linearize(position: UVec3) -> usize {
        let x = position.x as usize;
        let y = position.y as usize;
        let z = position.z as usize;

        x + (y * CHUNK_SIZE) + (z * CHUNK_SIZE * CHUNK_SIZE)
    }

    fn delinearize(mut index: usize) -> (u32, u32, u32) {
        let z = index / (CHUNK_SIZE * CHUNK_SIZE);
        index -= z * CHUNK_SIZE * CHUNK_SIZE;

        let y = index / CHUNK_SIZE;
        index -= y * CHUNK_SIZE;

        let x = index;

        (x as u32, y as u32, z as u32)
    }

    fn get(&self, x: u32, y: u32, z: u32) -> Self::Output;
}

#[derive(Component)]
pub struct ChunkBundle {
    pub position: ChunkPosition,
    pub data: BaseChunk,
    pub entities: Vec<Entity>,
}

const CHUNK_BORDER: u32 = (CHUNK_SIZE as u32) + 1;
const CHUNK_SIZE_WITH_NEIGHBORS: usize = CHUNK_SIZE + 2;
const CHUNK_VOLUME_WITH_NEIGHBORS: usize =
    CHUNK_SIZE_WITH_NEIGHBORS * CHUNK_SIZE_WITH_NEIGHBORS * CHUNK_SIZE_WITH_NEIGHBORS;

pub struct ChunkWithNeighbors {
    pub voxels: Box<[VoxelType; CHUNK_VOLUME_WITH_NEIGHBORS]>,
}

impl Chunk for ChunkWithNeighbors {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE_WITH_NEIGHBORS;
    const Y: usize = CHUNK_SIZE_WITH_NEIGHBORS;
    const Z: usize = CHUNK_SIZE_WITH_NEIGHBORS;

    fn get(&self, x: u32, y: u32, z: u32) -> Self::Output {
        self.voxels[Self::linearize(UVec3::new(x, y, z))]
    }
}

impl ChunkWithNeighbors {
    pub fn new(chunk: &BaseChunk, neighbors: &[BaseChunk; 26]) -> Self {
        const U32_CHUNK_SIZE: u32 = CHUNK_SIZE as u32;

        let voxels = Box::new(std::array::from_fn(|idx| {
            let (x, y, z) = ChunkWithNeighbors::delinearize(idx);

            match (x, y, z) {
                (0, 0, 0) => {
                    neighbors[0].get(U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1)
                }
                (0, 0, 1..=U32_CHUNK_SIZE) => {
                    neighbors[1].get(U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1, z - 1)
                }
                (0, 0, CHUNK_BORDER) => neighbors[2].get(U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1, 0),
                (0, 1..=U32_CHUNK_SIZE, 0) => {
                    neighbors[3].get(U32_CHUNK_SIZE - 1, y - 1, U32_CHUNK_SIZE - 1)
                }
                (0, 1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE) => {
                    neighbors[4].get(U32_CHUNK_SIZE - 1, y - 1, z - 1)
                }
                (0, 1..=U32_CHUNK_SIZE, CHUNK_BORDER) => {
                    neighbors[5].get(U32_CHUNK_SIZE - 1, y - 1, 0)
                }
                (0, CHUNK_BORDER, 0) => neighbors[6].get(U32_CHUNK_SIZE - 1, 0, U32_CHUNK_SIZE - 1),
                (0, CHUNK_BORDER, 1..=U32_CHUNK_SIZE) => {
                    neighbors[7].get(U32_CHUNK_SIZE - 1, 0, z - 1)
                }
                (0, CHUNK_BORDER, CHUNK_BORDER) => neighbors[8].get(U32_CHUNK_SIZE - 1, 0, 0),
                (1..=U32_CHUNK_SIZE, 0, 0) => {
                    neighbors[9].get(x - 1, U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1)
                }
                (1..=U32_CHUNK_SIZE, 0, 1..=U32_CHUNK_SIZE) => {
                    neighbors[10].get(x - 1, U32_CHUNK_SIZE - 1, z - 1)
                }
                (1..=U32_CHUNK_SIZE, 0, CHUNK_BORDER) => {
                    neighbors[11].get(x - 1, U32_CHUNK_SIZE - 1, 0)
                }
                (1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE, 0) => {
                    neighbors[12].get(x - 1, y - 1, U32_CHUNK_SIZE - 1)
                }
                (1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE) => {
                    chunk.get(x - 1, y - 1, z - 1)
                }
                (1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE, CHUNK_BORDER) => {
                    neighbors[13].get(x - 1, y - 1, 0)
                }
                (1..=U32_CHUNK_SIZE, CHUNK_BORDER, 0) => {
                    neighbors[14].get(x - 1, 0, U32_CHUNK_SIZE - 1)
                }
                (1..=U32_CHUNK_SIZE, CHUNK_BORDER, 1..=U32_CHUNK_SIZE) => {
                    neighbors[15].get(x - 1, 0, z - 1)
                }
                (1..=U32_CHUNK_SIZE, CHUNK_BORDER, CHUNK_BORDER) => neighbors[16].get(x - 1, 0, 0),
                (CHUNK_BORDER, 0, 0) => {
                    neighbors[17].get(0, U32_CHUNK_SIZE - 1, U32_CHUNK_SIZE - 1)
                }
                (CHUNK_BORDER, 0, 1..=U32_CHUNK_SIZE) => {
                    neighbors[18].get(0, U32_CHUNK_SIZE - 1, z - 1)
                }
                (CHUNK_BORDER, 0, CHUNK_BORDER) => neighbors[19].get(0, U32_CHUNK_SIZE - 1, 0),
                (CHUNK_BORDER, 1..=U32_CHUNK_SIZE, 0) => {
                    neighbors[20].get(0, y - 1, U32_CHUNK_SIZE - 1)
                }
                (CHUNK_BORDER, 1..=U32_CHUNK_SIZE, 1..=U32_CHUNK_SIZE) => {
                    neighbors[21].get(0, y - 1, z - 1)
                }
                (CHUNK_BORDER, 1..=U32_CHUNK_SIZE, CHUNK_BORDER) => neighbors[22].get(0, y - 1, 0),
                (CHUNK_BORDER, CHUNK_BORDER, 0) => neighbors[23].get(0, 0, U32_CHUNK_SIZE - 1),
                (CHUNK_BORDER, CHUNK_BORDER, 1..=U32_CHUNK_SIZE) => neighbors[24].get(0, 0, z - 1),
                (CHUNK_BORDER, CHUNK_BORDER, CHUNK_BORDER) => neighbors[25].get(0, 0, 0),

                (_, _, _) => VoxelType::default(),
            }
        }));

        Self { voxels }
    }
}

#[derive(Clone)]
pub struct BaseChunk(pub Box<[VoxelType; CHUNK_VOLUME]>);

impl Chunk for BaseChunk {
    type Output = VoxelType;

    const X: usize = CHUNK_SIZE;
    const Y: usize = CHUNK_SIZE;
    const Z: usize = CHUNK_SIZE;

    fn get(&self, x: u32, y: u32, z: u32) -> Self::Output {
        self.0[Self::linearize(UVec3::new(x, y, z))]
    }
}

impl BaseChunk {
    pub fn new() -> Self {
        Self(Box::new([VoxelType::default(); CHUNK_VOLUME]))
    }

    pub fn set(&mut self, position: UVec3, voxel: VoxelType) {
        self.0[Self::linearize(position)] = voxel;
    }
}
