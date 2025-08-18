use std::time::Instant;

use bevy::{asset::RenderAssetUsages, prelude::*, render::mesh::Indices};

use crate::{
    voxel::{MeshStats, Voxel},
    world::WorldManager,
};

#[derive(Component)]
pub struct NeedsRemesh;

#[derive(Component)]
pub struct NeedsDespawn;

#[derive(Component)]
pub struct Chunk {
    pub(crate) entity: Entity,
    pub(crate) position: IVec3,
}

impl Chunk {
    pub fn new(position: IVec3, entity: Entity) -> Self {
        Self { entity, position }
    }
}

#[derive(Clone)]
pub struct ChunkData {
    entity: Entity,
    pub position: IVec3,
    pub voxels: [Voxel; ChunkData::SIZE * ChunkData::SIZE * ChunkData::SIZE],
    pub dirty: bool,
}

impl ChunkData {
    pub const SIZE: usize = 32;

    pub fn new(position: IVec3) -> Self {
        Self {
            voxels: [Voxel::Air; Self::SIZE * Self::SIZE * Self::SIZE],
            position,
            dirty: false,
            entity: Entity::PLACEHOLDER,
        }
    }

    pub fn with_entity(position: IVec3, entity: Entity) -> ChunkData {
        let new = Self::new(position);
        Self { entity, ..new }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < Self::SIZE && y < Self::SIZE && z < Self::SIZE);
        x + y * Self::SIZE + z * Self::SIZE * Self::SIZE
    }

    pub fn get_voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set_voxel(&mut self, voxel: Voxel, x: usize, y: usize, z: usize) {
        self.voxels[Self::index(x, y, z)] = voxel;
        self.dirty = true;
    }

    pub fn generate_mesh(&self, world: &WorldManager) -> Mesh {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut colors: Vec<[f32; 4]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count: u32 = 0;

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                for z in 0..Self::SIZE {
                    let voxel = self.get_voxel(x, y, z);

                    if voxel.is_solid() {
                        self.add_exposed_faces(
                            &mut vertices,
                            &mut colors,
                            &mut indices,
                            &mut vertex_count,
                            x as i32,
                            y as i32,
                            z as i32,
                            voxel,
                            world,
                        );
                    }
                }
            }
        }

        self.create_bevy_mesh(vertices, colors, indices)
    }

    pub fn generate_mesh_with_stats(&self, world: &WorldManager) -> (Mesh, MeshStats) {
        let start = Instant::now();

        let mesh = self.generate_mesh(world);

        let stats = MeshStats {
            vertex_count: mesh.count_vertices(),
            triangle_count: mesh.triangles().iter().len(),
            generated_time_ms: start.elapsed().as_secs_f32() * 1000.0,
            algorithm: "Face Culling".to_string(),
            ..Default::default()
        };

        (mesh, stats)
    }

    fn create_bevy_mesh(
        &self,
        vertices: Vec<[f32; 3]>,
        colors: Vec<[f32; 4]>,
        indices: Vec<u32>,
    ) -> Mesh {
        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        // Set vertex locations in the world
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        // Set colors
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);

        // Set the triangle indices
        mesh.insert_indices(Indices::U32(indices));

        // Compute normals
        mesh.compute_normals();

        mesh
    }

    fn add_exposed_faces(
        &self,
        vertices: &mut Vec<[f32; 3]>,
        colors: &mut Vec<[f32; 4]>,
        indices: &mut Vec<u32>,
        vertex_count: &mut u32,
        x: i32,
        y: i32,
        z: i32,
        voxel: Voxel,
        world: &WorldManager,
    ) {
        let fx = x as f32;
        let fy = y as f32;
        let fz = z as f32;

        for (face_direction, offset) in FaceDirection::neighbor_offsets() {
            let x = x + offset.x;
            let y = y + offset.y;
            let z = z + offset.z;

            if !self.get_neighbor_voxel(x, y, z, world).is_solid() {
                self.add_cube_face_to_mesh(
                    vertices,
                    colors,
                    indices,
                    vertex_count,
                    fx,
                    fy,
                    fz,
                    face_direction,
                    voxel,
                );
            }
        }
    }

    fn get_neighbor_voxel(&self, x: i32, y: i32, z: i32, world: &WorldManager) -> Voxel {
        if x >= 0
            && y >= 0
            && z >= 0
            && x < Self::SIZE as i32
            && y < Self::SIZE as i32
            && z < Self::SIZE as i32
        {
            return self.get_voxel(x as usize, y as usize, z as usize);
        }

        let world_pos = IVec3::new(
            self.position.x * Self::SIZE as i32 + x,
            self.position.y * Self::SIZE as i32 + y,
            self.position.z * Self::SIZE as i32 + z,
        );

        world.get_voxel(&world_pos)
    }

    fn add_cube_face_to_mesh(
        &self,
        vertices: &mut Vec<[f32; 3]>,
        colors: &mut Vec<[f32; 4]>,
        indices: &mut Vec<u32>,
        vertex_count: &mut u32,
        x: f32,
        y: f32,
        z: f32,
        face_direction: FaceDirection,
        voxel: Voxel,
    ) {
        let face_verts = face_direction.face();

        let material = voxel.get_material();
        let color_array = material.color.to_srgba().to_f32_array();

        for vertex in face_verts.iter() {
            vertices.push([
                vertex[0] + x, // X coordinate + voxel X position
                vertex[1] + y, // Y coordinate + voxel Y position
                vertex[2] + z, // Z coordinate + voxel Z position
            ]);

            colors.push(color_array);
        }

        for &index in FACE_INDICES.iter() {
            indices.push(index + *vertex_count);
        }

        *vertex_count += 4;
    }

    pub fn get_world_transform(&self) -> Transform {
        Transform::from_xyz(
            self.position.x as f32 * Self::SIZE as f32,
            self.position.y as f32 * Self::SIZE as f32,
            self.position.z as f32 * Self::SIZE as f32,
        )
    }
}

#[derive(Clone, Copy)]
enum FaceDirection {
    PosX,
    NegX,
    PosY,
    NegY,
    PosZ,
    NegZ,
}

impl FaceDirection {
    fn face(&self) -> [[f32; 3]; 4] {
        match self {
            FaceDirection::PosX => FACE_VERTICES[0],
            FaceDirection::NegX => FACE_VERTICES[1],
            FaceDirection::PosY => FACE_VERTICES[2],
            FaceDirection::NegY => FACE_VERTICES[3],
            FaceDirection::PosZ => FACE_VERTICES[4],
            FaceDirection::NegZ => FACE_VERTICES[5],
        }
    }

    fn neighbor_offsets() -> impl Iterator<Item = (FaceDirection, IVec3)> {
        vec![
            (FaceDirection::PosX, IVec3::new(1, 0, 0)),
            (FaceDirection::NegX, IVec3::new(-1, 0, 0)),
            (FaceDirection::PosY, IVec3::new(0, 1, 0)),
            (FaceDirection::NegY, IVec3::new(0, -1, 0)),
            (FaceDirection::PosZ, IVec3::new(0, 0, 1)),
            (FaceDirection::NegZ, IVec3::new(0, 0, -1)),
        ]
        .into_iter()
    }
}

const FACE_INDICES: [u32; 6] = [0, 3, 1, 1, 3, 2];

const FACE_VERTICES: [[[f32; 3]; 4]; 6] = [
    // PosX face (right)
    [
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 0.0],
    ],
    // NegX face (left)
    [
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 1.0, 1.0],
        [0.0, 0.0, 1.0],
    ],
    // PosY face (top)
    [
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ],
    // NegY face (bottom)
    [
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    ],
    // PosZ face (front)
    [
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 0.0, 1.0],
    ],
    // NegZ face (back)
    [
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0],
    ],
];
