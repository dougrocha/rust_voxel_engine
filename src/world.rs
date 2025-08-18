use std::collections::VecDeque;

use bevy::{platform::collections::HashMap, prelude::*};

use crate::{
    camera::PlayerCamera,
    chunk::{Chunk, ChunkData, NeedsDespawn, NeedsRemesh},
    terrain::TerrainGenerator,
    voxel::Voxel,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldManager>()
            .init_resource::<WorldManagerSpawnBuffer>()
            .init_resource::<WorldManagerDespawnBuffer>()
            .add_systems(PreStartup, setup)
            .add_systems(
                PreUpdate,
                (
                    (spawn_chunks, tag_chunk_despawn).chain(),
                    (despawn_deleted_chunks, flush_chunk_buffers).chain(),
                ),
            )
            .add_systems(Update, spawn_meshes);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((WorldEntity, Visibility::default(), Transform::default()));
}

fn spawn_chunks(
    mut commands: Commands,
    mut spawn_buffer: ResMut<WorldManagerSpawnBuffer>,
    world_manager: Res<WorldManager>,
    player_camera: Query<&Transform, With<PlayerCamera>>,
    world_entity: Query<Entity, With<WorldEntity>>,
) {
    let world_entity = world_entity.single().unwrap();
    let camera = player_camera.single().unwrap();
    let cam_pos = camera.translation.as_ivec3();

    let render_distance: i32 = 8;
    let render_distance_squared = render_distance.pow(2);
    let radius = render_distance / 2;

    let mut chunks_deque: VecDeque<IVec3> =
        VecDeque::with_capacity(render_distance_squared as usize);

    let cam_chunk_pos = cam_pos / ChunkData::SIZE as i32;
    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                chunks_deque.push_back(cam_chunk_pos + IVec3::new(x, y, z));
            }
        }
    }

    while let Some(chunk_position) = chunks_deque.pop_front() {
        // check if chunk is in range and queue if needed

        if chunk_position.distance_squared(cam_chunk_pos) > render_distance_squared {
            continue;
        }

        let has_chunk = world_manager.contains_chunk(&chunk_position);

        if !has_chunk {
            // queue chunk to load
            let chunk_entity = commands.spawn(NeedsRemesh).id();
            commands.entity(world_entity).add_child(chunk_entity);

            let chunk = Chunk::new(chunk_position, chunk_entity);

            spawn_buffer.push((
                chunk_position,
                ChunkData::with_entity(chunk_position, chunk.entity),
            ));

            commands.entity(chunk.entity).try_insert((
                chunk,
                Transform::from_translation(
                    chunk_position.as_vec3() * ChunkData::SIZE as f32 - 1.0,
                ),
            ));
        }
    }
}

fn tag_chunk_despawn(
    mut commands: Commands,
    all_chunks: Query<(&Chunk, Option<&ViewVisibility>)>,
    player_camera: Query<&Transform, With<PlayerCamera>>,
) {
    let camera = player_camera.single().unwrap();
    let cam_pos = camera.translation.as_ivec3();

    let render_distance: i32 = 8;
    let render_distance_squared = render_distance.pow(2);

    let cam_chunk_pos = cam_pos / ChunkData::SIZE as i32;

    let chunk_to_remove = {
        let mut remove = Vec::with_capacity(100);

        // visibility is used when we want to only despawn when the user isnt looking at the chunk
        for (chunk, _visibility) in all_chunks.iter() {
            let dist_squared = chunk.position.distance_squared(cam_chunk_pos);

            if dist_squared > render_distance_squared + 1 {
                remove.push(chunk);
            }
        }

        remove
    };

    for chunk in chunk_to_remove {
        commands.entity(chunk.entity).try_insert(NeedsDespawn);
    }
}

fn despawn_deleted_chunks(
    mut commands: Commands,
    mut despawn_buffer: ResMut<WorldManagerDespawnBuffer>,
    world_manager: Res<WorldManager>,
    deleted_chunks: Query<(Entity, &Chunk), With<NeedsDespawn>>,
) {
    for (entity, chunk) in deleted_chunks.iter() {
        if world_manager.contains_chunk(&chunk.position) {
            commands.entity(entity).despawn();
            despawn_buffer.push(chunk.position);
        }
    }
}

fn flush_chunk_buffers(
    mut world_manager: ResMut<WorldManager>,
    spawn_buffer: Res<WorldManagerSpawnBuffer>,
    despawn_buffer: Res<WorldManagerDespawnBuffer>,
) {
    for (position, chunk_data) in spawn_buffer.iter() {
        world_manager.add_chunk(*position, chunk_data.clone());
    }

    for position in despawn_buffer.iter() {
        world_manager.chunks.remove(position);
    }
}

fn spawn_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    world_manager: Res<WorldManager>,
    spawn_buffer: Res<WorldManagerSpawnBuffer>,
) {
    for (_position, chunk) in spawn_buffer.iter() {
        let (mesh, stats) = chunk.generate_mesh_with_stats(&world_manager);

        dbg!(stats);

        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            })),
            chunk.get_world_transform(),
        ));
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldManagerSpawnBuffer(Vec<(IVec3, ChunkData)>);

#[derive(Resource, Deref, DerefMut, Default)]
pub struct WorldManagerDespawnBuffer(Vec<IVec3>);

#[derive(Component)]
pub struct WorldEntity;

#[derive(Default, Resource)]
pub struct WorldManager {
    chunks: HashMap<IVec3, ChunkData>,
}

impl WorldManager {
    pub fn add_chunk(&mut self, position: IVec3, chunk: ChunkData) {
        self.chunks.insert(position, chunk);
    }

    pub fn contains_chunk(&self, position: &IVec3) -> bool {
        self.chunks.contains_key(position)
    }

    pub fn get_chunk(&self, position: &IVec3) -> Option<&ChunkData> {
        self.chunks.get(position)
    }

    pub fn get_chunk_mut(&mut self, position: &IVec3) -> Option<&mut ChunkData> {
        self.chunks.get_mut(position)
    }

    fn world_to_chunk_pos(&self, world_pos: &IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.div_euclid(ChunkData::SIZE as i32),
            world_pos.y.div_euclid(ChunkData::SIZE as i32),
            world_pos.z.div_euclid(ChunkData::SIZE as i32),
        )
    }

    fn world_to_local_pos(&self, world_pos: &IVec3) -> IVec3 {
        IVec3::new(
            world_pos.x.rem_euclid(ChunkData::SIZE as i32),
            world_pos.y.rem_euclid(ChunkData::SIZE as i32),
            world_pos.z.rem_euclid(ChunkData::SIZE as i32),
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

    pub fn generate_world(&mut self, size: i32, seed: u32) {
        let terrain_generator = TerrainGenerator::new(seed);

        for x in 0..size {
            for z in 0..size {
                let chunk_pos = IVec3::new(x, 0, z);

                let chunk = self.generate_chunk(chunk_pos, &terrain_generator);

                self.add_chunk(chunk_pos, chunk);
            }
        }
    }

    fn generate_chunk(&self, chunk_pos: IVec3, terrain_generator: &TerrainGenerator) -> ChunkData {
        let mut chunk = ChunkData::new(chunk_pos);

        for x in 0..ChunkData::SIZE {
            for z in 0..ChunkData::SIZE {
                for y in 0..ChunkData::SIZE {
                    let voxel = terrain_generator.get_voxel(IVec3::new(
                        (chunk_pos.x * ChunkData::SIZE as i32) + x as i32,
                        (chunk_pos.y * ChunkData::SIZE as i32) + y as i32,
                        (chunk_pos.z * ChunkData::SIZE as i32) + z as i32,
                    ));
                    chunk.set_voxel(voxel, x, y, z);
                }
            }
        }

        chunk
    }
}
