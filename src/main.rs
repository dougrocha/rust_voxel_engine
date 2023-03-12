pub mod world;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    tasks::{AsyncComputeTaskPool, Task},
};

use bevy_voxel_game::{
    player::{MovementSettings, Player, PlayerPlugin},
    world::{
        chunk::{
            chunks::Chunk, create_chunk, destroy_chunks, mesh::create_chunk_mesh, ChunkEntities,
            ChunkPosition, ChunkQueue, CHUNK_SIZE,
        },
        world::{ChunkMap, ViewDistance},
    },
};
use futures_lite::future;

const CLEAR_COLOR: Color = Color::rgb(0.4, 0.4, 0.4);

fn main() {
    App::new()
        .insert_resource(ClearColor(CLEAR_COLOR))
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Voxel Engine".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(MovementSettings {
            walk_speed: 30.0,
            ..default()
        })
        // Game State
        .add_plugin(WorldGenerationPlugin)
        .add_startup_system(setup_light)
        .add_startup_system(setup_world)
        .add_plugin(PlayerPlugin)
        .run();
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

#[derive(Resource)]
struct GameSettings {
    view_distance: u32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self { view_distance: 12 }
    }
}

fn update_chunks(
    mut chunk_queue: ResMut<ChunkQueue>,
    player: Query<&Transform, With<Player>>,
    chunk_entities: ResMut<ChunkEntities>,
    game_settings: Res<GameSettings>,
) {
    let player_pos = player.single().translation;
    let player_chunk = ChunkPosition::from_world_position(Vec3 {
        x: player_pos.x,
        y: player_pos.y,
        z: player_pos.z,
    });

    let view_distance = game_settings.view_distance as i32;

    for x in -view_distance..=view_distance {
        for z in -view_distance..=view_distance {
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
        if (loaded_chunk.x - player_chunk.x).abs() > view_distance
            || (loaded_chunk.z - player_chunk.z).abs() > view_distance
        {
            chunk_queue.remove.push(*loaded_chunk);
        }
    }
}

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ChunkMap::new())
            .init_resource::<ChunkEntities>()
            .init_resource::<ChunkQueue>()
            .init_resource::<GameSettings>()
            .add_systems(
                (
                    update_chunks,
                    create_chunk,
                    generate_chunk,
                    render_chunk,
                    destroy_chunks,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
struct ChunkMeshTask(Task<(Mesh, ChunkPosition)>);

pub fn generate_chunk(
    mut commands: Commands,
    chunk_entities: ResMut<ChunkEntities>,
    mut chunk_map: ResMut<ChunkMap>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    // generate each entity
    for (chunk_pos, chunk_entity) in chunk_entities.iter() {
        chunk_map.set(*chunk_pos, Chunk::new(*chunk_pos));

        let chunk = chunk_map.get(chunk_pos).unwrap();

        let chunk_position = *chunk_pos;

        let task = thread_pool.spawn(async move {
            let chunk_mesh = create_chunk_mesh(&chunk);

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, chunk_mesh.vertices);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, chunk_mesh.normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, chunk_mesh.uvs);

            mesh.set_indices(Some(Indices::U32(chunk_mesh.indices)));

            (mesh, chunk_position)
        });

        commands
            .get_entity(*chunk_entity)
            .unwrap()
            .insert(ChunkMeshTask(task));
    }
}

fn render_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_tasks: Query<(Entity, &mut ChunkMeshTask)>,
) {
    for (entity, mut task) in &mut mesh_tasks {
        if let Some((mesh, chunk_position)) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).insert((
                PbrBundle {
                    mesh: meshes.add(mesh),
                    material: materials.add(StandardMaterial {
                        base_color: Color::GREEN,
                        ..Default::default()
                    }),
                    transform: Transform::from_translation(Vec3::new(
                        chunk_position.x as f32 * CHUNK_SIZE as f32,
                        0.0,
                        chunk_position.z as f32 * CHUNK_SIZE as f32,
                    )),
                    ..Default::default()
                },
                // Only render the wireframe of the mesh for testing purposes
                // Wireframe,
            ));
        }
    }
}
