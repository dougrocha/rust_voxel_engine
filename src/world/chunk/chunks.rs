// Code ported from https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/

// Note this implementation does not support different block types or block normals
// The original author describes how to do this here: https://0fps.net/2012/07/07/meshing-minecraft-part-2/
// https://gist.github.com/Vercidium/a3002bd083cce2bc854c9ff8f0118d33
// https://gist.github.com/IcyTv/7408523d21b8cccbfb29f927d4e098b2

use bevy::{
    pbr::wireframe::Wireframe,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use noise::NoiseFn;

use crate::world::{
    block::{Block, BlockPosition, BlockType, Direction},
    voxel_data::{FACES, FACE_ORDER, NORMALS, UVS, VERTICES},
};

use super::{ChunkArray, ChunkPosition, FMask, CHUNK_HEIGHT, CHUNK_SIZE};

#[derive(Component)]
pub struct Chunk {
    pub chunk_array: ChunkArray,
    pub position: ChunkPosition,
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,

    noise: noise::SuperSimplex,

    vertex_count: u32,

    pub mesh: Mesh,
}

impl Chunk {
    pub fn new(position: ChunkPosition) -> Chunk {
        Chunk {
            chunk_array: ChunkArray::new(),
            position,
            vertices: Vec::new(),
            indices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            noise: noise::SuperSimplex::new(123),
            vertex_count: 0,
            mesh: Mesh::new(PrimitiveTopology::TriangleList),
        }
    }

    pub fn get_block(&self, position: BlockPosition) -> Option<&Block> {
        if position.x < 0
            || position.x >= CHUNK_SIZE
            || position.y < 0
            || position.y >= CHUNK_HEIGHT
            || position.z < 0
            || position.z >= CHUNK_SIZE
        {
            return None;
        }

        Some(
            &self.chunk_array.blocks[position.x as usize][position.y as usize][position.z as usize],
        )
    }

    pub fn set_block(&mut self, position: BlockPosition, block: Block) {
        if position.x < 0
            || position.x >= CHUNK_SIZE
            || position.y < 0
            || position.y >= CHUNK_HEIGHT
            || position.z < 0
            || position.z >= CHUNK_SIZE
        {
            return;
        }

        self.chunk_array.blocks[position.x as usize][position.y as usize][position.z as usize] =
            block;
    }

    pub fn render(
        &mut self,
        commands: &mut Commands,
        materials: &mut ResMut<Assets<StandardMaterial>>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        self.generate_greedy_mesh();
        // self.generate_mesh();

        self.apply_mesh();

        commands.spawn((
            PbrBundle {
                mesh: meshes.add(self.mesh.clone()),
                material: materials.add(StandardMaterial {
                    base_color: Color::GREEN,
                    ..Default::default()
                }),
                transform: Transform::from_translation(Vec3::new(
                    self.position.x as f32 * CHUNK_SIZE as f32,
                    0.0,
                    self.position.z as f32 * CHUNK_SIZE as f32,
                )),
                ..Default::default()
            },
            // Only render the wireframe of the mesh for testing purposes
            // Wireframe,
        ));
    }

    pub fn generate_blocks(&mut self) {
        for x in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let height = self.noise.get([
                    (self.position.x * CHUNK_SIZE + x) as f64 / 16.0,
                    (self.position.z * CHUNK_SIZE + z) as f64 / 16.0,
                ]) * 5.0
                    + 64.0;

                for y in 0..CHUNK_HEIGHT {
                    let block_position = BlockPosition::new(x, y, z);

                    if (y as f64) <= height {
                        self.set_block(block_position, Block::new(BlockType::DIRT));
                    } else {
                        self.set_block(block_position, Block::new(BlockType::AIR));
                    }
                }
            }
        }
    }

    pub fn generate_greedy_mesh(&mut self) {
        // Iterate through each axis (X,Y,Z)

        let chunk_sizes = [CHUNK_SIZE, CHUNK_HEIGHT, CHUNK_SIZE];

        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;

            let mut x = [0, 0, 0];
            let mut q = [0, 0, 0];

            let mask_size = chunk_sizes[u] * chunk_sizes[v];
            let mut mask: Vec<FMask> = vec![FMask::new(BlockType::AIR, 0); mask_size as usize];

            q[d] = 1;

            // Check each slice of the chunk one at a time
            x[d] = -1;
            while x[d] < chunk_sizes[d] {
                let mut n = 0i32;

                // Compute the mask

                x[v] = 0;
                while x[v] < chunk_sizes[v] {
                    x[u] = 0;
                    while x[u] < chunk_sizes[u] {
                        // q determines the direction (X, Y or Z) that we are searching
                        // check_block will check if the next block is transparent

                        let mut current_block: bool = false;
                        if 0 <= x[d] {
                            current_block = !self.check_block(BlockPosition::new(x[0], x[1], x[2]));
                        }

                        let mut next_block: bool = false;
                        if x[d] < chunk_sizes[d] - 1 && x[d] + q[d] >= 0 {
                            next_block = !self.check_block(BlockPosition::new(
                                x[0] + q[0],
                                x[1] + q[1],
                                x[2] + q[2],
                            ));
                        }

                        // The mask is set to true if there is a visible face between two blocks,
                        //   i.e. both aren't empty and both aren't blocks
                        mask[n as usize] = if current_block == next_block {
                            FMask::new(BlockType::AIR, 0)
                        } else if current_block {
                            FMask::new(
                                self.get_block(BlockPosition::new(x[0], x[1], x[2]))
                                    .unwrap()
                                    .block_type,
                                1,
                            )
                        } else {
                            FMask::new(
                                self.get_block(BlockPosition::new(
                                    x[0] + q[0],
                                    x[1] + q[1],
                                    x[2] + q[2],
                                ))
                                .unwrap()
                                .block_type,
                                -1,
                            )
                        };

                        n += 1;

                        x[u] += 1;
                    }

                    x[v] += 1;
                }

                x[d] += 1;

                n = 0;

                // Generate a mesh from the mask using lexicographic ordering,
                //   by looping over each block in this slice of the chunk

                for j in 0i32..chunk_sizes[v] {
                    let mut i = 0;
                    while i < chunk_sizes[u] {
                        if mask[n as usize].normal != 0 {
                            let current_mask: FMask = mask[n as usize];
                            // Compute the width of this quad and store it in w
                            //   This is done by searching along the current axis until mask[n + w] is false
                            let mut w = 1i32;

                            while i + w < chunk_sizes[u]
                                && Chunk::compare_mask(current_mask, mask[(n + w) as usize])
                            {
                                w += 1;
                            }

                            // Compute the height of this quad and store it in h
                            //   This is done by checking if every block next to this row (range 0 to w) is also part of the mask.
                            //   For example, if w is 5 we currently have a quad of dimensions 1 x 5. To reduce triangle count,
                            //   greedy meshing will attempt to expand this quad out to CHUNK_SIZE x 5, but will stop if it reaches a hole in the mask

                            let mut h = 1;
                            'outer: while (j + h as i32) < chunk_sizes[v] {
                                for k in 0..w {
                                    let compare_mask: FMask =
                                        mask[n as usize + k as usize + h * chunk_sizes[u] as usize];
                                    // if there is a hole in the mask, we can't expand the quad out any further
                                    if Chunk::compare_mask(current_mask, compare_mask) {
                                        continue;
                                    }
                                    break 'outer;
                                }

                                h += 1;
                            }

                            x[u] = i;
                            x[v] = j;

                            // du and  dv are the dimensions of the quad
                            let mut du = [0, 0, 0];
                            let mut dv = [0, 0, 0];

                            // Set the dimensions of the quad
                            du[u] = w;
                            dv[v] = h;

                            self.create_quad(
                                mask[n as usize],
                                q,
                                w as f32,
                                h as f32,
                                [x[0] as f32, x[1] as f32, x[2] as f32],
                                [
                                    (x[0] + du[0] as i32) as f32,
                                    (x[1] + du[1] as i32) as f32,
                                    (x[2] + du[2] as i32) as f32,
                                ],
                                [
                                    (x[0] + dv[0] as i32) as f32,
                                    (x[1] + dv[1] as i32) as f32,
                                    (x[2] + dv[2] as i32) as f32,
                                ],
                                [
                                    (x[0] + du[0] as i32 + dv[0] as i32) as f32,
                                    (x[1] + du[1] as i32 + dv[1] as i32) as f32,
                                    (x[2] + du[2] as i32 + dv[2] as i32) as f32,
                                ],
                            );

                            // Clear this part of the mask so that we don't create a quad for this face again
                            for l in 0..h {
                                for k in 0..w {
                                    mask[(n as usize + k as usize + l * chunk_sizes[u] as usize)] =
                                        FMask::new(BlockType::AIR, 0);
                                }
                            }

                            // Increment i and n by the width of the quad
                            i += w;
                            n += w;
                        } else {
                            i += 1;
                            n += 1;
                        }
                    }
                }
            }
        }
    }

    fn compare_mask(mask: FMask, compare: FMask) -> bool {
        mask.normal == compare.normal && mask.block_type == compare.block_type
    }

    pub fn create_quad(
        &mut self,
        mask: FMask,
        q: [i32; 3],
        width: f32,
        height: f32,
        top_left: [f32; 3],
        top_right: [f32; 3],
        bottom_right: [f32; 3],
        bottom_left: [f32; 3],
    ) {
        //  multiply q by mask.normal
        let normal = [
            q[0] * mask.normal as i32,
            q[1] * mask.normal as i32,
            q[2] * mask.normal as i32,
        ];

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();

        //  top left
        vertices.push([top_left[0] as f32, top_left[1] as f32, top_left[2] as f32]);

        //  top right
        vertices.push([
            top_right[0] as f32,
            top_right[1] as f32,
            top_right[2] as f32,
        ]);

        //  bottom right
        vertices.push([
            bottom_right[0] as f32,
            bottom_right[1] as f32,
            bottom_right[2] as f32,
        ]);

        //  bottom left
        vertices.push([
            bottom_left[0] as f32,
            bottom_left[1] as f32,
            bottom_left[2] as f32,
        ]);

        // indices
        indices.push(self.vertex_count);
        indices.push((self.vertex_count + 2).wrapping_sub(mask.normal as u32));
        indices.push((self.vertex_count + 2).wrapping_add(mask.normal as u32));
        indices.push(self.vertex_count + 3);
        indices.push((self.vertex_count + 1).wrapping_add(mask.normal as u32));
        indices.push((self.vertex_count + 1).wrapping_sub(mask.normal as u32));

        // normals
        normals.push([normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        normals.push([normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        normals.push([normal[0] as f32, normal[1] as f32, normal[2] as f32]);
        normals.push([normal[0] as f32, normal[1] as f32, normal[2] as f32]);

        if normal[0] == 1 || normal[0] == -1 {
            uvs.push([width, height]);
            uvs.push([0.0, height]);
            uvs.push([width, 0.0]);
            uvs.push([0.0, 0.0]);
        } else {
            uvs.push([width, height]);
            uvs.push([height, 0.0]);
            uvs.push([0.0, width]);
            uvs.push([0.0, 0.0]);
        }

        self.vertices.append(&mut vertices);
        self.indices.append(&mut indices);
        self.normals.append(&mut normals);
        self.uvs.append(&mut uvs);

        self.vertex_count += 4;
    }

    pub fn generate_mesh(&mut self) {
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_HEIGHT {
                for z in 0..CHUNK_SIZE {
                    let block_position = BlockPosition::new(x, y, z);

                    if self.get_block(block_position).unwrap().block_type == BlockType::AIR {
                        continue;
                    }

                    for direction in 0..6 {
                        let direction = match direction {
                            0 => Direction::UP,
                            1 => Direction::DOWN,
                            2 => Direction::LEFT,
                            3 => Direction::RIGHT,
                            4 => Direction::FRONT,
                            5 => Direction::BACK,
                            _ => panic!("Invalid direction"),
                        };

                        let position_in_direction =
                            self.get_position_in_direction(block_position, direction);

                        if self.check_block(position_in_direction) {
                            self.create_face(block_position, direction);
                        }
                    }
                }
            }
        }
    }

    pub fn apply_mesh(&mut self) {
        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices.clone());
        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());
        self.mesh
            .insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs.clone());

        self.mesh
            .set_indices(Some(Indices::U32(self.indices.clone())));
    }

    pub fn create_face(&mut self, position: BlockPosition, direction: Direction) {
        self.set_face_verticies(position, direction);

        UVS.iter().for_each(|uv| {
            self.uvs.push(*uv);
        });

        // indices
        for index in 0..6 {
            self.indices.push(self.vertex_count + FACE_ORDER[index]);
        }

        self.vertex_count += 4;
    }

    pub fn get_normal(&self, direction: Direction) -> [f32; 3] {
        NORMALS[direction as usize]
    }

    pub fn set_face_verticies(&mut self, position: BlockPosition, direction: Direction) {
        for index in FACES[direction as usize] {
            let vertex = [
                position.x as f32 + VERTICES[index][0],
                position.y as f32 + VERTICES[index][1],
                position.z as f32 + VERTICES[index][2],
            ];

            self.vertices.push(vertex);

            let normal = self.get_normal(direction);
            self.normals.push(normal);
        }
    }

    /**
     * Checks if the block is transparent
     *
     * @param position The position of the block
     * @return True if the block is transparent
     */
    pub fn check_block(&self, position: BlockPosition) -> bool {
        if position.x < 0
            || position.x >= CHUNK_SIZE as i32
            || position.y < 0
            || position.y >= CHUNK_HEIGHT as i32
            || position.z < 0
            || position.z >= CHUNK_SIZE as i32
        {
            return true;
        }

        return self.get_block(position).unwrap().is_transparent();
    }

    pub fn get_position_in_direction(
        &self,
        position: BlockPosition,
        direction: Direction,
    ) -> BlockPosition {
        match direction {
            Direction::UP => BlockPosition::new(position.x, position.y + 1, position.z),
            Direction::DOWN => BlockPosition::new(position.x, position.y - 1, position.z),
            Direction::LEFT => BlockPosition::new(position.x - 1, position.y, position.z),
            Direction::RIGHT => BlockPosition::new(position.x + 1, position.y, position.z),
            Direction::FRONT => BlockPosition::new(position.x, position.y, position.z + 1),
            Direction::BACK => BlockPosition::new(position.x, position.y, position.z - 1),
        }
    }
}
