use crate::{Chunk, Visibility, EMPTY, OPAQUE, TRANSPARENT};

pub mod face;
pub mod side;

const CHUNK_SIZE: usize = 16;

#[derive(Copy, Clone, Debug)]
pub struct Quad {
    pub voxel: [usize; 3],
    pub width: u32,
    pub height: u32,
}

#[derive(Default)]
pub struct QuadGroups {
    pub groups: [Vec<Quad>; 6],
}

pub fn generate_mesh(chunk: &Chunk) -> QuadGroups {
    let mut buffer = QuadGroups::default();

    generate_mesh_buffer(chunk, &mut buffer);

    buffer
}

pub fn generate_mesh_buffer(chunk: &Chunk, buffer: &mut QuadGroups) {
    buffer.clear();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel = chunk.get(x, y, z);

                if (x > 0 && x < CHUNK_SIZE - 1)
                    && (y > 0 && y < CHUNK_SIZE - 1)
                    && (z > 0 && z < CHUNK_SIZE - 1)
                {
                    match voxel.visibility() {
                        Visibility::Empty => continue,
                        visibility => {
                            let neighbors = [
                                chunk.get(x - 1, y, z),
                                chunk.get(x + 1, y, z),
                                chunk.get(x, y - 1, z),
                                chunk.get(x, y + 1, z),
                                chunk.get(x, y, z - 1),
                                chunk.get(x, y, z + 1),
                            ];

                            for (i, neighbor) in neighbors.into_iter().enumerate() {
                                let other = neighbor.visibility();

                                let generate = match (visibility, other) {
                                    (OPAQUE, EMPTY)
                                    | (OPAQUE, TRANSPARENT)
                                    | (TRANSPARENT, EMPTY) => true,

                                    (TRANSPARENT, TRANSPARENT) => voxel != neighbor,

                                    (_, _) => false,
                                };

                                if generate {
                                    buffer.groups[i].push(Quad {
                                        voxel: [x, y, z],
                                        width: 1,
                                        height: 1,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
