use bevy::{platform::collections::HashMap, prelude::*};

use crate::{chunk::Chunk, voxel::Voxel};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<World>();
    }
}

#[derive(Default, Resource)]
pub struct World {
    chunks: HashMap<IVec3, Chunk>,
}

impl World {
    pub fn add_chunk(&mut self, position: IVec3, chunk: Chunk) {
        self.chunks.insert(position, chunk);
    }

    pub fn get_chunk(&self, position: &IVec3) -> Option<&Chunk> {
        self.chunks.get(position)
    }

    pub fn get_chunk_mut(&mut self, position: &IVec3) -> Option<&mut Chunk> {
        self.chunks.get_mut(position)
    }

    fn world_to_chunk_pos(&self, world_pos: &IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.div_euclid(Chunk::SIZE as i32),
            world_pos.y.div_euclid(Chunk::SIZE as i32),
            world_pos.z.div_euclid(Chunk::SIZE as i32),
        )
    }

    fn world_to_local_pos(&self, world_pos: &IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(Chunk::SIZE as i32),
            world_pos.y.rem_euclid(Chunk::SIZE as i32),
            world_pos.z.rem_euclid(Chunk::SIZE as i32),
        )
    }

    pub fn get_voxel(&self, world_pos: &IVec3) -> Voxel {
        let chunk_pos = self.world_to_chunk_pos(world_pos);
        let local_pos = self.world_to_local_pos(world_pos);

        if let Some(chunk) = self.get_chunk(&chunk_pos) {
            chunk.get_voxel(
                local_pos.x as usize,
                local_pos.y as usize,
                local_pos.z as usize,
            )
        } else {
            Voxel::Air // No chunk exists here
        }
    }
}
