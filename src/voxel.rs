use bevy::{
    asset::RenderAssetUsages,
    math::IVec3,
    render::mesh::{Indices, Mesh},
};

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Voxel {
    Air,
    Dirt,
    Grass,
    Stone,
}

impl Voxel {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Voxel::Air)
    }
}

pub struct Chunk {
    pub voxels: [Voxel; Chunk::SIZE * Chunk::SIZE * Chunk::SIZE],
    pub position: IVec3,
    pub dirty: bool,
}

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
}

impl Chunk {
    const SIZE: usize = 32;

    pub fn new(position: IVec3) -> Self {
        Self {
            voxels: [Voxel::Air; Self::SIZE * Self::SIZE * Self::SIZE],
            position,
            dirty: false,
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        debug_assert!(x < Self::SIZE && y < Self::SIZE && z < Self::SIZE);
        x + y * Self::SIZE + z * Self::SIZE * Self::SIZE
    }

    fn get_voxel(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[Self::index(x, y, z)]
    }

    pub fn set_voxel(&mut self, voxel: Voxel, x: usize, y: usize, z: usize) {
        self.voxels[Self::index(x, y, z)] = voxel;
        self.dirty = true;
    }

    pub fn generate_mesh(&self) -> Mesh {
        let mut vertices: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut vertex_count: u32 = 0;

        for x in 0..Self::SIZE {
            for y in 0..Self::SIZE {
                for z in 0..Self::SIZE {
                    let voxel = self.get_voxel(x, y, z);

                    if voxel.is_solid() {
                        self.add_exposed_faces(
                            &mut vertices,
                            &mut indices,
                            &mut vertex_count,
                            x as i32,
                            y as i32,
                            z as i32,
                        );
                    }
                }
            }
        }

        self.create_bevy_mesh(vertices, indices)
    }

    fn create_bevy_mesh(&self, vertices: Vec<[f32; 3]>, indices: Vec<u32>) -> Mesh {
        let mut mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );

        // Set vertex locations in the world
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

        // Set the triangle indices
        mesh.insert_indices(Indices::U32(indices));

        // Compute normals
        mesh.compute_normals();

        mesh
    }

    fn add_exposed_faces(
        &self,
        vertices: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u32>,
        vertex_count: &mut u32,
        x: i32,
        y: i32,
        z: i32,
    ) {
        let fx = x as f32;
        let fy = y as f32;
        let fz = z as f32;

        if !self.get_neighbor_voxel(x + 1, y, z).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::PosX,
            );
        }
        if !self.get_neighbor_voxel(x - 1, y, z).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::NegX,
            );
        }
        if !self.get_neighbor_voxel(x, y + 1, z).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::PosY,
            );
        }
        if !self.get_neighbor_voxel(x, y - 1, z).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::NegY,
            );
        }
        if !self.get_neighbor_voxel(x, y, z + 1).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::PosZ,
            );
        }
        if !self.get_neighbor_voxel(x, y, z - 1).is_solid() {
            self.add_cube_face_to_mesh(
                vertices,
                indices,
                vertex_count,
                fx,
                fy,
                fz,
                FaceDirection::NegZ,
            );
        }
    }

    fn get_neighbor_voxel(&self, x: i32, y: i32, z: i32) -> Voxel {
        if x as usize >= Self::SIZE
            || y as usize >= Self::SIZE
            || z as usize >= Self::SIZE
            || x < 0
            || y < 0
            || z < 0
        {
            // TODO: Handle checking neighbor chunks when world manager is setup
            return Voxel::Air;
        }

        self.get_voxel(x as usize, y as usize, z as usize)
    }

    fn add_cube_face_to_mesh(
        &self,
        vertices: &mut Vec<[f32; 3]>,
        indices: &mut Vec<u32>,
        vertex_count: &mut u32,
        x: f32,
        y: f32,
        z: f32,
        face_direction: FaceDirection,
    ) {
        let face_verts = face_direction.face();

        for vertex in face_verts.iter() {
            vertices.push([
                vertex[0] + x, // X coordinate + voxel X position
                vertex[1] + y, // Y coordinate + voxel Y position
                vertex[2] + z, // Z coordinate + voxel Z position
            ]);
        }

        for &index in FACE_INDICES.iter() {
            indices.push(index + *vertex_count);
        }

        *vertex_count += 4;
    }
}
