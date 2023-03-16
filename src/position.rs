use bevy::prelude::{IVec2, IVec3, Vec3};

use crate::chunk::CHUNK_SIZE;

pub fn world_to_chunk(world_position: &Vec3) -> IVec3 {
    IVec3::new(
        (world_position.x / CHUNK_SIZE as f32).floor() as i32,
        (world_position.y / CHUNK_SIZE as f32).floor() as i32,
        (world_position.z / CHUNK_SIZE as f32).floor() as i32,
    )
}

pub fn positions_in_radius(radius: i32) -> Vec<IVec2> {
    let center = IVec2::new(0, 0);
    let mut positions = Vec::new();

    for x in -radius..=radius {
        for z in -radius..=radius {
            let curr_pos = IVec2::new(x, z);
            let distance_squared = curr_pos.as_vec2().distance_squared(center.as_vec2()) as i32;

            if distance_squared <= radius.pow(2) {
                positions.push(curr_pos);
            }
        }
    }

    positions
}
