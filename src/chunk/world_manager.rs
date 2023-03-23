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

use super::ChunkPosition;

#[derive(SystemParam)]
pub struct WorldManager<'w, 's> {
    pub world: ResMut<'w, World>,
    pub render_distance: Res<'w, RenderDistance>,
    pub chunks: Query<'w, 's, &'static ChunkBundle>,
}

impl<'w, 's> WorldManager<'w, 's> {
    pub fn chunks_positions_in_render_distance(
        &self,
        chunk_position: ChunkPosition,
    ) -> Vec<ChunkPosition> {
        let mut chunks = Vec::new();

        for position in positions_in_radius(self.render_distance.horizontal) {
            for y in -self.render_distance.vertical..=self.render_distance.vertical {
                let position = chunk_position.0 + IVec3::new(position.x, y, position.y);
                chunks.push(ChunkPosition(position));
            }
        }

        chunks.sort_unstable_by_key(|key| {
            (key.x - chunk_position.0.x).abs() + (key.0.z - chunk_position.0.z).abs()
        });

        chunks
    }

    pub fn chunks_in_render_distance(
        &mut self,
        chunk_position: ChunkPosition,
    ) -> Vec<&ChunkBundle> {
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

    pub fn neighboring_chunks(&self, chunk_position: ChunkPosition) -> Vec<BaseChunk> {
        chunk_position
            .neighbors()
            .iter()
            .map(|pos| self.world.get_entity(*pos))
            .flatten()
            .map(|entity| self.chunks.get(entity))
            .flatten()
            .map(|chunk| chunk.data.clone())
            .collect()
    }
}
