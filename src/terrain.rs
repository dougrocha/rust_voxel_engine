use bevy::math::IVec3;
use noise::{HybridMulti, NoiseFn, Perlin};

use crate::voxel::Voxel;

pub enum Biome {
    Plains,
    Desert,
}

pub struct TerrainGenerator {
    height_noise: HybridMulti<Perlin>,
    temperature_noise: HybridMulti<Perlin>,
    base_height: f64,
}

impl TerrainGenerator {
    pub fn new(seed: u32) -> Self {
        let mut height_noise = HybridMulti::<Perlin>::new(seed);
        height_noise.octaves = 5;
        height_noise.frequency = 1.1;
        height_noise.lacunarity = 2.8;
        height_noise.persistence = 0.4;

        let mut temperature_noise = HybridMulti::<Perlin>::new(seed + 1000);
        temperature_noise.octaves = 3;
        temperature_noise.frequency = 0.8;
        temperature_noise.lacunarity = 2.0;
        temperature_noise.persistence = 0.5;

        Self {
            height_noise,
            temperature_noise,
            base_height: 5.,
        }
    }

    fn get_biome(&self, world_pos: IVec3) -> Biome {
        let temperature = self
            .temperature_noise
            .get([world_pos.x as f64 / 500.0, world_pos.z as f64 / 500.0]);

        match temperature {
            t if t > 0.2 => Biome::Desert,
            _ => Biome::Plains,
        }
    }

    pub fn get_voxel(&self, world_pos: IVec3) -> Voxel {
        let IVec3 { x, y, z } = world_pos;

        let biome = self.get_biome(world_pos);

        let height =
            self.base_height + self.height_noise.get([x as f64 / 100.0, z as f64 / 100.0]) * 20.0;

        let depth_below_surface = height - (y as f64);

        if (y as f64) <= height {
            match biome {
                Biome::Plains => {
                    if depth_below_surface < 1.0 {
                        Voxel::Grass
                    } else if depth_below_surface < 4.0 {
                        Voxel::Dirt
                    } else {
                        Voxel::Stone
                    }
                }
                Biome::Desert => {
                    if depth_below_surface < 1.0 {
                        Voxel::Sand
                    } else if depth_below_surface < 6.0 {
                        Voxel::SandStone
                    } else {
                        Voxel::Stone
                    }
                }
            }
        } else {
            Voxel::Air
        }
    }
}
