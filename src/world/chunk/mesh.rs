use crate::world::block::{BlockPosition, BlockType};

use super::{chunks::Chunk, CHUNK_HEIGHT, CHUNK_SIZE};

#[derive(Clone, Copy)]
pub struct FMask {
    pub block_type: BlockType,
    pub normal: i8,
}

impl FMask {
    pub fn new(block_type: BlockType, normal: i8) -> FMask {
        FMask { block_type, normal }
    }
}

#[derive(Default)]
pub struct ChunkMesh {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u32>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,

    pub vertex_count: u32,
}

pub fn create_chunk_mesh(chunk: &Chunk) -> ChunkMesh {
    let mut mesh: ChunkMesh = ChunkMesh::default();

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
                        current_block = !chunk.check_block(BlockPosition::new(x[0], x[1], x[2]));
                    }

                    let mut next_block: bool = false;
                    if x[d] < chunk_sizes[d] - 1 && x[d] + q[d] >= 0 {
                        next_block = !chunk.check_block(BlockPosition::new(
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
                            chunk
                                .get_block(BlockPosition::new(x[0], x[1], x[2]))
                                .unwrap()
                                .block_type,
                            1,
                        )
                    } else {
                        FMask::new(
                            chunk
                                .get_block(BlockPosition::new(
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
                            && compare_mask(current_mask, mask[(n + w) as usize])
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
                                let mask_to_compare: FMask =
                                    mask[n as usize + k as usize + h * chunk_sizes[u] as usize];
                                // if there is a hole in the mask, we can't expand the quad out any further
                                if compare_mask(current_mask, mask_to_compare) {
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

                        create_quad(
                            &mut mesh,
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
                                    FMask::new(BlockType::DEFAULT, 0);
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

    mesh
}

fn create_quad(
    mesh: &mut ChunkMesh,
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
    indices.push(mesh.vertex_count);
    indices.push((mesh.vertex_count + 2).wrapping_sub(mask.normal as u32));
    indices.push((mesh.vertex_count + 2).wrapping_add(mask.normal as u32));
    indices.push(mesh.vertex_count + 3);
    indices.push((mesh.vertex_count + 1).wrapping_add(mask.normal as u32));
    indices.push((mesh.vertex_count + 1).wrapping_sub(mask.normal as u32));

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

    mesh.vertices.append(&mut vertices);
    mesh.indices.append(&mut indices);
    mesh.normals.append(&mut normals);
    mesh.uvs.append(&mut uvs);

    mesh.vertex_count += 4;
}

fn compare_mask(mask: FMask, compare: FMask) -> bool {
    mask.normal == compare.normal && mask.block_type == compare.block_type
}
