use super::{components::Chunk, CHUNK_SIZE};

pub const EMPTY: Visibility = Visibility::Empty;
pub const OPAQUE: Visibility = Visibility::Opaque;
pub const TRANSPARENT: Visibility = Visibility::Transparent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Empty,
    Transparent,
    Opaque,
}

pub trait Voxel: Eq {
    fn visibility(&self) -> Visibility;
}

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

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub enum VoxelType {
    #[default]
    Empty,
    Opaque(u32),
    Transparent(u32),
}

impl Voxel for VoxelType {
    fn visibility(&self) -> Visibility {
        match self {
            Self::Empty => Visibility::Empty,
            Self::Opaque(_) => Visibility::Opaque,
            Self::Transparent(_) => Visibility::Transparent,
        }
    }
}

pub fn generate_mesh<C, T>(chunk: &C) -> QuadGroups
where
    C: Chunk<Output = T>,
    T: Voxel,
{
    let mut buffer = QuadGroups::default();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let x = x as u32;
                let y = y as u32;
                let z = z as u32;

                if (x > 0 && x < (CHUNK_SIZE as u32) - 1)
                    && (y > 0 && y < (CHUNK_SIZE as u32) - 1)
                    && (z > 0 && z < (CHUNK_SIZE as u32) - 1)
                {
                    let voxel = chunk.get(x, y, z);

                    match voxel.visibility() {
                        EMPTY => continue,
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
                                        voxel: [x as usize, y as usize, z as usize],
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

    buffer
}

pub enum Axis {
    X,
    Y,
    Z,
}

pub struct Side {
    pub axis: Axis,
    pub positive: bool,
}

impl Side {
    pub fn new(axis: Axis, positive: bool) -> Self {
        Self { axis, positive }
    }

    pub fn normal(&self) -> [f32; 3] {
        match (&self.axis, &self.positive) {
            (Axis::X, true) => [1.0, 0.0, 0.0],   // X+
            (Axis::X, false) => [-1.0, 0.0, 0.0], // X-
            (Axis::Y, true) => [0.0, 1.0, 0.0],   // Y+
            (Axis::Y, false) => [0.0, -1.0, 0.0], // Y-
            (Axis::Z, true) => [0.0, 0.0, 1.0],   // Z+
            (Axis::Z, false) => [0.0, 0.0, -1.0], // Z-
        }
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        [self.normal(), self.normal(), self.normal(), self.normal()]
    }
}

pub struct Face<'a> {
    side: Side,
    quad: &'a Quad,
}

impl<'a> Face<'a> {
    pub fn indices(&self, start: u32) -> [u32; 6] {
        [start, start + 2, start + 1, start + 1, start + 2, start + 3]
    }

    pub fn positions(&self, voxel_size: f32) -> [[f32; 3]; 4] {
        let positions = match (&self.side.axis, &self.side.positive) {
            (Axis::X, false) => [
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            (Axis::X, true) => [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            (Axis::Y, false) => [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
            (Axis::Y, true) => [
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            (Axis::Z, false) => [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            (Axis::Z, true) => [
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 1.0, 1.0],
            ],
        };

        let (x, y, z) = (
            (self.quad.voxel[0] - 1) as f32,
            (self.quad.voxel[1] - 1) as f32,
            (self.quad.voxel[2] - 1) as f32,
        );

        [
            [
                x * voxel_size + positions[0][0] * voxel_size,
                y * voxel_size + positions[0][1] * voxel_size,
                z * voxel_size + positions[0][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[1][0] * voxel_size,
                y * voxel_size + positions[1][1] * voxel_size,
                z * voxel_size + positions[1][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[2][0] * voxel_size,
                y * voxel_size + positions[2][1] * voxel_size,
                z * voxel_size + positions[2][2] * voxel_size,
            ],
            [
                x * voxel_size + positions[3][0] * voxel_size,
                y * voxel_size + positions[3][1] * voxel_size,
                z * voxel_size + positions[3][2] * voxel_size,
            ],
        ]
    }

    pub fn normals(&self) -> [[f32; 3]; 4] {
        self.side.normals()
    }

    pub fn uvs(&self, flip_u: bool, flip_v: bool) -> [[f32; 2]; 4] {
        match (flip_u, flip_v) {
            (true, true) => [[1.0, 1.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
            (true, false) => [[1.0, 0.0], [0.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            (false, true) => [[0.0, 1.0], [1.0, 1.0], [0.0, 0.0], [1.0, 0.0]],
            (false, false) => [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        }
    }

    pub fn voxel(&self) -> [usize; 3] {
        self.quad.voxel
    }
}

// For our new `iter()` method
// Note: it is important this matches the neighbor ordering from `simple_mesh`
impl From<usize> for Side {
    fn from(value: usize) -> Self {
        match value {
            0 => Self::new(Axis::X, false), // X-
            1 => Self::new(Axis::X, true),  // X+
            2 => Self::new(Axis::Y, false), // Y-
            3 => Self::new(Axis::Y, true),  // Y+
            4 => Self::new(Axis::Z, false), // Z-
            5 => Self::new(Axis::Z, true),  // Z+
            _ => unreachable!(),
        }
    }
}

impl QuadGroups {
    pub fn iter(&self) -> impl Iterator<Item = Face> {
        self.groups
            .iter()
            .enumerate()
            .flat_map(|(index, quads)| quads.iter().map(move |quad| (index, quad)))
            .map(|(index, quad)| Face {
                side: index.into(),
                quad,
            })
    }
}
