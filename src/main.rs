mod player;

use std::sync::{Arc, Mutex};

use bevy::{prelude::*, tasks::Task, utils::hashbrown::HashMap};
use player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .run();
}

fn setup() {
    println!("Hello, world!");
}

struct World {
    chunks: HashMap<IVec3, Arc<Mutex<Chunk>>>,
}

struct Chunk {
    voxels: Vec<Voxel>,
}

struct Voxel;

#[derive(Component)]
struct ChunkGenerateQueue(Task<IVec3>);

#[derive(Component)]
struct ChunkRenderQueue(Task<(Chunk, Mesh)>);
