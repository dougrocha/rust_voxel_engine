use bevy::prelude::*;

#[derive(Default, Debug)]
pub struct MeshStats {
    pub triangle_count: usize,
    pub vertex_count: usize,
    pub generated_time_ms: f32,
    pub solid_voxel_count: usize,
    pub algorithm: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Voxel {
    Air,
    Dirt,
    Grass,
    Stone,
}

pub struct VoxelMaterial {
    pub color: Color,
}

impl Voxel {
    pub fn is_solid(&self) -> bool {
        !matches!(self, Voxel::Air)
    }

    pub fn get_material(&self) -> VoxelMaterial {
        match self {
            Voxel::Air => VoxelMaterial { color: Color::NONE },
            Voxel::Dirt => VoxelMaterial {
                color: Color::srgb(0.55, 0.27, 0.07),
            },
            Voxel::Grass => VoxelMaterial {
                color: Color::srgb(0.34, 0.69, 0.31),
            },
            Voxel::Stone => VoxelMaterial {
                color: Color::srgb(0.60, 0.60, 0.60),
            },
        }
    }
}
