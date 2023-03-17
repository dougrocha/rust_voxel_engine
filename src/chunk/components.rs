use std::fmt::Debug;

use bevy::{
    prelude::{Component, Entity, IVec3, UVec3, Vec3},
    reflect::Reflect,
};

use super::{
    mesh::{Voxel, VoxelType},
    CHUNK_SIZE, CHUNK_VOLUME,
};

#[derive(Component)]
pub struct AwaitingMesh;

#[derive(Component)]
pub struct DestroyChunk;

pub trait Chunk {
    type Output;

    const X: usize = CHUNK_SIZE;
    const Y: usize = CHUNK_SIZE;
    const Z: usize = CHUNK_SIZE;

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
pub struct BaseChunk {
    pub position: IVec3,
    pub voxels: VoxelContainer,
    pub entities: Vec<Entity>,
}

impl Chunk for BaseChunk {
    type Output = VoxelType;

    fn get(&self, x: u32, y: u32, z: u32) -> Self::Output {
        self.voxels.0[Self::linearize(UVec3::new(x, y, z))]
    }
}

impl BaseChunk {
    pub fn set(&mut self, position: UVec3, voxel: VoxelType) {
        self.voxels.0[Self::linearize(position)] = voxel;
    }
}

#[derive(Clone)]
pub struct VoxelContainer(pub Box<[VoxelType; CHUNK_VOLUME]>);

impl VoxelContainer {
    pub fn new() -> Self {
        Self(Box::new([VoxelType::default(); CHUNK_VOLUME]))
    }
}
