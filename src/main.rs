use std::collections::BTreeMap;

use bevy::{
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::schedule::ShouldRun,
    pbr::wireframe::WireframePlugin,
    prelude::*,
    utils::HashMap,
};

use bevy_inspector_egui::{
    bevy_egui::EguiContext, widgets::InNewWindow, Inspectable, InspectorPlugin,
    WorldInspectorPlugin,
};

use ndshape::{ConstShape, ConstShape3i32, ConstShape3u32};
use noise::NoiseFn;
use rust_game::{
    player::{MovementSettings, Player, PlayerPlugin, PlayerPosition},
    world::{
        block::{Block, BlockPosition, BlockType},
        chunk::{
            chunks::Chunk, create_chunk, destroy_chunks, ChunkEntities, ChunkPosition, ChunkQueue,
            CHUNK_HEIGHT, CHUNK_SIZE,
        },
        world::{ChunkMap, ViewDistance},
    },
};

const CLEAR_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR_COLOR))
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Voxel Engine".into(),
                ..default()
            },
            ..default()
        }))
        .insert_resource(MovementSettings {
            walk_speed: 30.0,
            ..default()
        })
        // Debug
        .add_plugin(DebugUIPlugin)
        // Game State
        .add_plugin(WorldGenerationPlugin)
        .add_startup_system(setup_light)
        .add_startup_system(setup_world)
        .add_plugin(PlayerPlugin)
        .run();
}

/// Run criteria for the [`update_view_chunks`] system
fn update_chunks_criteria(
    position: Res<PlayerPosition>,
    view_distance: Res<ViewDistance>,
) -> ShouldRun {
    if position.is_changed() || view_distance.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, StageLabel)]
pub struct ChunkLoadingStage;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, SystemLabel)]
/// Labels for the systems added by [`VoxelWorldChunkingPlugin`]
pub enum ChunkLoadingSystem {
    /// Updates the player current chunk.
    /// The computed position is used for loading / meshing priority systems.
    UpdatePlayerPos,
    /// Runs chunk view distance calculations and queue events for chunk creations and deletions.
    UpdateViewChunks,
    /// Creates the voxel buffers to hold chunk data and attach them a chunk entity in the ECS world.
    CreateChunks,
    /// Clears the dirty chunks list.
    ClearDirtyChunks,

    // Generate the mesh for the chunk.
    GenerateMesh,
}

/// set up a simple 3D scene
fn setup_light(mut commands: Commands) {
    //sky light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.80,
    });
}

fn setup_world(
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut chunk_queue: ResMut<ChunkQueue>,
) {
    // add one chunk to queue

    //     let mut world = ChunkWorld::new();

    //     world.render(commands, meshes, materials);
}

fn update_player_position(
    player: Query<&GlobalTransform, (With<Player>, Changed<GlobalTransform>)>,
    mut player_pos: ResMut<PlayerPosition>,
) {
    if let Ok(ply) = player.get_single() {
        let player_coords = ply.translation().as_ivec3();
        player_pos.x = player_coords.x as f32;
        player_pos.y = player_coords.y as f32;
        player_pos.z = player_coords.z as f32;
    }
}

fn update_chunks(
    mut chunk_queue: ResMut<ChunkQueue>,
    player_pos: ResMut<PlayerPosition>,
    chunk_entities: ResMut<ChunkEntities>,
    view_distance: Res<ViewDistance>,
) {
    let player_chunk = ChunkPosition::from_world_position(Vec3 {
        x: player_pos.x,
        y: player_pos.y,
        z: player_pos.z,
    });

    for x in -view_distance.distance..=view_distance.distance {
        for z in -view_distance.distance..=view_distance.distance {
            let chunk_pos = ChunkPosition {
                x: player_chunk.x + x,
                z: player_chunk.z + z,
            };

            if !chunk_entities.contains_key(&chunk_pos) {
                chunk_queue.create.push(chunk_pos);
            }
        }
    }

    // remove chunks that are too far away
    for loaded_chunk in chunk_entities.iter_keys() {
        if (loaded_chunk.x - player_chunk.x).abs() > view_distance.distance
            || (loaded_chunk.z - player_chunk.z).abs() > view_distance.distance
        {
            chunk_queue.remove.push(*loaded_chunk);
        }
    }
}

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkMap::new())
            .insert_resource(ViewDistance { distance: 5 })
            .init_resource::<ChunkEntities>()
            .insert_resource(PlayerPosition {
                x: 0.0,
                y: 100.0,
                z: 0.0,
            })
            .init_resource::<ChunkQueue>()
            .add_stage_after(
                CoreStage::Update,
                ChunkLoadingStage,
                SystemStage::parallel()
                    .with_system(update_player_position.label(ChunkLoadingSystem::UpdatePlayerPos))
                    .with_system(
                        update_chunks
                            .label(ChunkLoadingSystem::UpdateViewChunks)
                            .after(ChunkLoadingSystem::UpdatePlayerPos)
                            .with_run_criteria(update_chunks_criteria),
                    )
                    .with_system(
                        create_chunk
                            .label(ChunkLoadingSystem::CreateChunks)
                            .after(ChunkLoadingSystem::UpdateViewChunks),
                    )
                    .with_system(
                        generate_chunk
                            .label(ChunkLoadingSystem::GenerateMesh)
                            .after(ChunkLoadingSystem::CreateChunks)
                            .with_run_criteria(generate_chunk_criteria),
                    ),
            )
            .add_system_to_stage(CoreStage::Last, destroy_chunks);
    }
}

pub fn generate_chunk_criteria(chunk_entities: Res<ChunkEntities>) -> ShouldRun {
    if chunk_entities.is_changed() {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

pub fn generate_chunk(
    mut commands: Commands,
    chunk_entities: ResMut<ChunkEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_map: ResMut<ChunkMap>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // generate each entity
    for chunk_pos in chunk_entities.iter_keys() {
        println!("Generating chunk {:?}", chunk_pos);
        chunk_map.set(*chunk_pos, Chunk::new(*chunk_pos));

        let chunk = chunk_map.get_mut(&*chunk_pos).unwrap();

        chunk.render(&mut commands, &mut materials, &mut meshes);
    }
}

pub struct DebugUIPlugin;

impl Plugin for DebugUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(WorldInspectorPlugin::default())
            .add_plugin(LogDiagnosticsPlugin::default())
            .add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(EntityCountDiagnosticsPlugin::default())
            .add_plugin(WireframePlugin)
            .add_plugin(InspectorPlugin::<PlayerPosition>::new());
    }
}
