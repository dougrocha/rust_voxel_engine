use bevy::{
    math::Vec3Swizzles,
    prelude::{Entity, IVec3, Resource},
    utils::hashbrown::HashMap,
};
use rand::Rng;
use tokio::sync::mpsc::{Receiver, Sender};

use super::components::BaseChunk;

#[derive(Default, Resource)]
pub struct PlayerChunk(pub IVec3);

impl PlayerChunk {
    pub fn set(&mut self, position: IVec3) {
        self.0 = position;
    }
}

#[derive(Resource)]
pub struct ChunkChannel(pub (Sender<BaseChunk>, Receiver<BaseChunk>));

#[derive(Default, Resource)]
// Will be used for chunk generation
pub struct ChunkQueue {
    pub generate: Vec<IVec3>,
    pub await_mesh: Vec<(IVec3, BaseChunk)>,
}

/**
 * Will be random by default
 */
#[derive(Resource)]
pub struct WorldSeed(pub u32);

impl Default for WorldSeed {
    fn default() -> Self {
        Self(rand::thread_rng().gen_range(0..u32::MAX))
    }
}

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<IVec3, Entity>,
}

impl World {
    pub fn get_entity(&self, position: IVec3) -> Option<Entity> {
        self.chunks.get(&position).copied()
    }

    pub fn add_chunk(&mut self, position: IVec3, entity: Entity) {
        self.chunks.insert(position, entity);
    }

    pub fn remove_chunk(&mut self, position: &IVec3) {
        self.chunks.remove(position);
    }

    pub fn check_neighbors(&self, position: IVec3) -> bool {
        let mut neighbors = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    neighbors.push(position + IVec3::new(x, y, z));
                }
            }
        }
        neighbors.iter().all(|pos| self.chunks.contains_key(pos))
    }

    pub fn get_neighbors(&self, position: IVec3) -> Vec<Entity> {
        let mut neighbors = Vec::new();
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    if x == 0 && y == 0 && z == 0 {
                        continue;
                    }
                    if let Some(entity) = self.chunks.get(&(position + IVec3::new(x, y, z))) {
                        neighbors.push(*entity);
                    }
                }
            }
        }
        neighbors
    }

    pub fn can_render(
        center: &IVec3,
        chunk_position: &IVec3,
        render_distance: &RenderDistance,
    ) -> bool {
        !(chunk_position
            .xz()
            .as_vec2()
            .distance(center.xz().as_vec2())
            .abs()
            .floor() as i32
            > render_distance.horizontal
            || (chunk_position.y - center.y).abs() > render_distance.vertical)
    }
}

#[derive(Resource)]
pub struct RenderDistance {
    pub horizontal: i32,
    pub vertical: i32,
}
