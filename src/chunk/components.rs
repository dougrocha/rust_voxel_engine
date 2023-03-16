use bevy::prelude::{Component, Entity, IVec3};

use super::CHUNK_VOLUME;

#[derive(Component)]
pub struct AwaitingMesh;

#[derive(Component)]
pub struct DestroyChunk;

#[derive(Component)]
pub struct Chunk {
    pub position: IVec3,
    pub voxels: VoxelContainer,
    pub entities: Vec<Entity>,
}

#[derive(Clone)]
pub struct VoxelContainer(Box<[u16; CHUNK_VOLUME]>);

impl VoxelContainer {
    pub fn new() -> Self {
        Self(Box::new([0; CHUNK_VOLUME]))
    }
}
