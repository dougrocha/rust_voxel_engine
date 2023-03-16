pub mod components;
mod resources;
mod systems;

use bevy::{math::Vec3Swizzles, prelude::*, utils::hashbrown::HashMap};
use rand::Rng;

use self::resources::{ChunkQueue, PlayerChunk};
use self::systems::{
    chunk_generation_poll, destroy_chunk_poll, destroy_chunks, generate_chunk, generate_mesh,
    should_load_chunks, update_player_chunk,
};

pub const CHUNK_SIZE: u32 = 16;
pub const CHUNK_VOLUME: usize =
    (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize) * (CHUNK_SIZE as usize);

#[derive(Default, Resource)]
pub struct WorldSeed(pub u64);

#[derive(Resource, Default)]
pub struct World {
    pub chunks: HashMap<IVec3, Entity>,
}

#[derive(Resource)]
pub struct RenderDistance {
    pub horizontal: i32,
    pub vertical: i32,
}

pub struct ChunkPlugin;

impl Plugin for ChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>()
            .init_resource::<ChunkQueue>()
            .init_resource::<PlayerChunk>()
            .insert_resource(RenderDistance {
                horizontal: 3,
                vertical: 3,
            })
            .insert_resource(WorldSeed(rand::thread_rng().gen_range(0..u64::MAX)))
            .add_systems((
                update_player_chunk,
                chunk_generation_poll,
                generate_chunk,
                generate_mesh,
                destroy_chunk_poll.run_if(should_load_chunks),
                destroy_chunks,
            ));
    }
}

impl World {
    pub fn get_entity(&self, position: IVec3) -> Option<Entity> {
        self.chunks.get(&position).copied()
    }

    pub fn add_chunk(&mut self, position: IVec3, entity: Entity) {
        self.chunks.insert(position, entity);
    }

    pub fn remove_chunk(&mut self, position: &IVec3) {
        self.chunks.remove(&position);
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
