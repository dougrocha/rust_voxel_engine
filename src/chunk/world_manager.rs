use bevy::{
    ecs::system::SystemParam,
    prelude::{IVec3, Query, Res, ResMut},
};

use crate::{
    chunk::{
        components::{BaseChunk, ChunkBundle},
        resources::{RenderDistance, World},
    },
    position::positions_in_radius,
};

#[derive(SystemParam)]
pub struct WorldManager<'w, 's> {
    pub world: ResMut<'w, World>,
    pub render_distance: Res<'w, RenderDistance>,
    pub chunks: Query<'w, 's, &'static ChunkBundle>,
}

impl<'w, 's> WorldManager<'w, 's> {
    pub fn chunks_positions_in_render_distance(&self, chunk_position: IVec3) -> Vec<IVec3> {
        let mut chunks = Vec::new();

        for position in positions_in_radius(self.render_distance.horizontal) {
            for y in -self.render_distance.vertical..=self.render_distance.vertical {
                chunks.push(chunk_position + IVec3::new(position.x, y, position.y));
            }
        }

        chunks.sort_unstable_by_key(|key| {
            (key.x - chunk_position.x).abs() + (key.z - chunk_position.z).abs()
        });

        chunks
    }

    pub fn chunks_in_render_distance(&mut self, chunk_position: IVec3) -> Vec<&ChunkBundle> {
        let mut chunks = Vec::new();

        for position in self
            .chunks_positions_in_render_distance(chunk_position)
            .iter()
        {
            if let Some(entity) = self.world.get_entity(*position) {
                if let Ok(chunk) = self.chunks.get(entity) {
                    chunks.push(chunk);
                }
            }
        }

        chunks
    }

    pub fn neighboring_chunks(&self, chunk_position: IVec3) -> Option<Vec<BaseChunk>> {
        let mut chunks = Vec::new();

        for entity in self.world.get_neighbors(chunk_position) {
            if let Ok(chunk) = self.chunks.get(entity) {
                chunks.push(chunk.data.clone());
            }
        }

        Some(chunks)
    }
}
