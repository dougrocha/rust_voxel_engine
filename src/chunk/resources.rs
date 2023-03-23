use bevy::{
    math::Vec3Swizzles,
    prelude::{Entity, IVec3, Mesh, Resource},
    utils::hashbrown::HashMap,
};
use rand::Rng;
use rayon::prelude::*;
use tokio::sync::mpsc::{Receiver, Sender};

use super::{
    components::{BaseChunk, ChunkBundle, ChunkWithNeighbors},
    ChunkPosition,
};

#[derive(Default, Resource)]
pub struct PlayerChunk(pub IVec3);

impl PlayerChunk {
    pub fn set(&mut self, position: IVec3) {
        self.0 = position;
    }
}

#[derive(Resource)]
pub struct ChunkChannel(pub (Sender<ChunkBundle>, Receiver<ChunkBundle>));

pub struct MeshedChunk {
    pub mesh: Mesh,
    pub position: ChunkPosition,
}

#[derive(Resource)]
pub struct MeshChannel(pub (Sender<MeshedChunk>, Receiver<MeshedChunk>));

#[derive(Default, Resource)]
// Will be used for chunk generation
pub struct ChunkQueue {
    pub generate: Vec<ChunkPosition>,
    pub await_mesh: Vec<(ChunkPosition, BaseChunk, Box<[BaseChunk; 26]>)>,
}

/**
 * Will be random by default
 */
#[derive(Resource)]
pub struct WorldSeed(pub u64);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(rand::thread_rng().gen_range(0..u64::MAX))
    }
}

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<ChunkPosition, Entity>,
}

impl World {
    pub fn get_entity(&self, position: ChunkPosition) -> Option<Entity> {
        self.chunks.get(&position).copied()
    }

    pub fn add_chunk(&mut self, position: ChunkPosition, entity: Entity) {
        self.chunks.insert(position, entity);
    }

    pub fn remove_chunk(&mut self, position: &ChunkPosition) {
        self.chunks.remove(position);
    }

    pub fn check_neighbors(&self, position: ChunkPosition) -> bool {
        position
            .neighbors()
            .iter()
            .all(|pos| self.chunks.contains_key(pos))
    }

    pub fn get_neighbors(&self, position: ChunkPosition) -> Vec<Entity> {
        position
            .neighbors()
            .iter()
            .filter_map(|pos| self.chunks.get(pos))
            .copied()
            .collect()
    }

    pub fn can_render(
        center: &IVec3,
        chunk_position: &ChunkPosition,
        render_distance: &RenderDistance,
    ) -> bool {
        !(chunk_position
            .0
            .xz()
            .as_vec2()
            .distance(center.xz().as_vec2())
            .abs()
            .floor() as i32
            > render_distance.horizontal
            || (chunk_position.0.y - center.y).abs() > render_distance.vertical)
    }
}

#[derive(Resource)]
pub struct RenderDistance {
    pub horizontal: i32,
    pub vertical: i32,
}
