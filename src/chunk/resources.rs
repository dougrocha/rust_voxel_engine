use bevy::prelude::{IVec3, Resource};

#[derive(Default, Resource)]
pub struct PlayerChunk(pub IVec3);

impl PlayerChunk {
    pub fn set(&mut self, position: IVec3) {
        self.0 = position;
    }
}

#[derive(Default, Resource)]
// Will be used for chunk generation
pub struct ChunkQueue {
    pub generate: Vec<IVec3>,
}
