use bevy::prelude::*;

pub mod camera;
pub mod debug;
pub mod world;

const DEFAULT_RENDER_LAYER: usize = 0;
const VIEW_MODEL_RENDER_LAYER: usize = 1;

#[derive(Component)]
pub struct Player;
