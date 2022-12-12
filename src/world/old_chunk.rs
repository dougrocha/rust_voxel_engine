// use bevy::{
//     prelude::Mesh,
//     render::{mesh::Indices, render_resource::PrimitiveTopology},
// };

// use super::{
//     block::{Block, BlockType},
//     voxel_data::*,
// };

// pub struct ChunkPosition {
//     pub x: i32,
//     pub y: i32,
//     pub z: i32,
// }

// impl ChunkPosition {
//     pub fn new(x: i32, y: i32, z: i32) -> ChunkPosition {
//         ChunkPosition { x, y, z }
//     }
// }

// pub type ChunkArray = [[[Block; CHUNK_SIZE as usize]; CHUNK_HEIGHT as usize]; CHUNK_SIZE as usize];

// pub struct Chunk {
//     pub blocks: ChunkArray,
//     pub position: ChunkPosition,
//     pub mesh: Mesh,
// }

// impl Chunk {
//     pub fn new(position: ChunkPosition) -> Chunk {
//         let blocks = [[[Block::new(BlockType::DEFAULT); CHUNK_HEIGHT as usize];
//             CHUNK_SIZE as usize]; CHUNK_SIZE as usize];

//         let mesh = Mesh::new(PrimitiveTopology::TriangleList);

//         Chunk {
//             blocks,
//             mesh,
//             position,
//         }
//     }
// }

// pub struct GreedyMesh {
//     mesh: Mesh,
//     vertices: Vec<[f32; 3]>,
//     indices: Vec<u32>,
//     normals: Vec<[f32; 3]>,
//     i: u32,
// }

// impl GreedyMesh {
//     pub fn new() -> GreedyMesh {
//         let mesh = Mesh::new(PrimitiveTopology::TriangleList);

//         let vertices: Vec<[f32; 3]> = Vec::new();
//         let indices: Vec<u32> = Vec::new();
//         let normals: Vec<[f32; 3]> = Vec::new();

//         let i = 0;

//         GreedyMesh {
//             mesh,
//             vertices,
//             indices,
//             normals,
//             i,
//         }
//     }

//     pub fn

//     pub fn add_face(&mut self, face: Face, x: usize, y: usize, z: usize) {
//         let f: usize = face as usize;

//         let normal = NORMALS[f];

//         for vertex in 0..4 {
//             let vertex_index = FACES[f][vertex];

//             let vertex = VERTICES[vertex_index];

//             let vertex = [
//                 vertex[0] + x as f32,
//                 vertex[1] + y as f32,
//                 vertex[2] + z as f32,
//             ];

//             self.vertices.push(vertex);
//             self.normals.push(normal);
//         }

//         for index in 0..6 {
//             self.indices
//                 .push((FACE_ORDER[index] + self.i as usize).try_into().unwrap());
//         }

//         self.i += 4;
//     }

//     pub fn render(&mut self) -> Mesh {
//         self.mesh
//             .insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices.clone());
//         self.mesh
//             .insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());

//         self.mesh
//             .set_indices(Some(Indices::U32(self.indices.clone())));

//         self.mesh.clone()
//     }
// }
