pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    // pub uv: [f32; 2],
}

pub struct GreedyMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    // pub texture_atlas: TextureAtlas,
}

impl GreedyMesh {
    pub fn new(// texture_atlas: TextureAtlas
    ) -> GreedyMesh {
        GreedyMesh {
            vertices: Vec::new(),
            indices: Vec::new(),
            // texture_atlas,
        }
    }
}
