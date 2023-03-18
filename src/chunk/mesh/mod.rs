use std::ops::Deref;

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

    generate_mesh_buffer(chunk, &mut buffer);

    buffer
}

pub fn generate_mesh_buffer<C, T>(chunk: &C, buffer: &mut QuadGroups)
where
    C: Chunk<Output = T>,
    T: Voxel,
{
    assert!(C::X >= 2);
    assert!(C::Y >= 2);
    assert!(C::Z >= 2);

    buffer.clear();

    for x in 0..C::X {
        for y in 0..C::Y {
            for z in 0..C::Z {
                let x = x as u32;
                let y = y as u32;
                let z = z as u32;

                if (x > 0 && x < (C::X as u32) - 1)
                    && (y > 0 && y < (C::Y as u32) - 1)
                    && (z > 0 && z < (C::Z as u32) - 1)
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
}

pub enum Axis {
    XPositive,
    XNegative,
    YPositive,
    YNegative,
    ZPositive,
    ZNegative,
}

pub struct Side {
    pub axis: Axis,
}

impl Side {
    pub fn new(axis: Axis) -> Self {
        Self { axis }
    }

    pub fn normal(&self) -> [f32; 3] {
        match &self.axis {
            Axis::XPositive => [1.0, 0.0, 0.0],  // X+
            Axis::XNegative => [-1.0, 0.0, 0.0], // X-
            Axis::YPositive => [0.0, 1.0, 0.0],  // Y+
            Axis::YNegative => [0.0, -1.0, 0.0], // Y-
            Axis::ZPositive => [0.0, 0.0, 1.0],  // Z+
            Axis::ZNegative => [0.0, 0.0, -1.0], // Z-
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
        let positions = match &self.side.axis {
            Axis::XNegative => [
                [0.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            Axis::XPositive => [
                [1.0, 0.0, 0.0],
                [1.0, 0.0, 1.0],
                [1.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            Axis::YNegative => [
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 1.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
            ],
            Axis::YPositive => [
                [0.0, 1.0, 1.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 0.0],
            ],
            Axis::ZNegative => [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            Axis::ZPositive => [
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
            0 => Self::new(Axis::XNegative), // X-
            1 => Self::new(Axis::XPositive), // X+
            2 => Self::new(Axis::YNegative), // Y-
            3 => Self::new(Axis::YPositive), // Y+
            4 => Self::new(Axis::ZNegative), // Z-
            5 => Self::new(Axis::ZPositive), // Z+
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

    pub fn clear(&mut self) {
        self.groups.iter_mut().for_each(|g| g.clear());
    }

    pub fn iter_with_ao<'a, C, V>(&'a self, chunk: &'a C) -> impl Iterator<Item = FaceWithAO<'a>>
    where
        C: Chunk<Output = V>,
        V: Voxel,
    {
        self.iter().map(|face| FaceWithAO::new(face, chunk))
    }
}

pub(crate) fn face_aos<C, V>(face: &Face, chunk: &C) -> [u32; 4]
where
    C: Chunk<Output = V>,
    V: Voxel,
{
    let [x, y, z] = face.voxel();

    let x = x as u32;
    let y = y as u32;
    let z = z as u32;

    match face.side.axis {
        Axis::XNegative => side_aos([
            chunk.get(x - 1, y, z + 1),
            chunk.get(x - 1, y - 1, z + 1),
            chunk.get(x - 1, y - 1, z),
            chunk.get(x - 1, y - 1, z - 1),
            chunk.get(x - 1, y, z - 1),
            chunk.get(x - 1, y + 1, z - 1),
            chunk.get(x - 1, y + 1, z),
            chunk.get(x - 1, y + 1, z + 1),
        ]),
        Axis::XPositive => side_aos([
            chunk.get(x + 1, y, z - 1),
            chunk.get(x + 1, y - 1, z - 1),
            chunk.get(x + 1, y - 1, z),
            chunk.get(x + 1, y - 1, z + 1),
            chunk.get(x + 1, y, z + 1),
            chunk.get(x + 1, y + 1, z + 1),
            chunk.get(x + 1, y + 1, z),
            chunk.get(x + 1, y + 1, z - 1),
        ]),
        Axis::YNegative => side_aos([
            chunk.get(x - 1, y - 1, z),
            chunk.get(x - 1, y - 1, z + 1),
            chunk.get(x, y - 1, z + 1),
            chunk.get(x + 1, y - 1, z + 1),
            chunk.get(x + 1, y - 1, z),
            chunk.get(x + 1, y - 1, z - 1),
            chunk.get(x, y - 1, z - 1),
            chunk.get(x - 1, y - 1, z - 1),
        ]),
        Axis::YPositive => side_aos([
            chunk.get(x, y + 1, z + 1),
            chunk.get(x - 1, y + 1, z + 1),
            chunk.get(x - 1, y + 1, z),
            chunk.get(x - 1, y + 1, z - 1),
            chunk.get(x, y + 1, z - 1),
            chunk.get(x + 1, y + 1, z - 1),
            chunk.get(x + 1, y + 1, z),
            chunk.get(x + 1, y + 1, z + 1),
        ]),
        Axis::ZNegative => side_aos([
            chunk.get(x - 1, y, z - 1),
            chunk.get(x - 1, y - 1, z - 1),
            chunk.get(x, y - 1, z - 1),
            chunk.get(x + 1, y - 1, z - 1),
            chunk.get(x + 1, y, z - 1),
            chunk.get(x + 1, y + 1, z - 1),
            chunk.get(x, y + 1, z - 1),
            chunk.get(x - 1, y + 1, z - 1),
        ]),
        Axis::ZPositive => side_aos([
            chunk.get(x + 1, y, z + 1),
            chunk.get(x + 1, y - 1, z + 1),
            chunk.get(x, y - 1, z + 1),
            chunk.get(x - 1, y - 1, z + 1),
            chunk.get(x - 1, y, z + 1),
            chunk.get(x - 1, y + 1, z + 1),
            chunk.get(x, y + 1, z + 1),
            chunk.get(x + 1, y + 1, z + 1),
        ]),
    }
}

pub struct FaceWithAO<'a> {
    face: Face<'a>,
    aos: [u32; 4],
}

impl<'a> FaceWithAO<'a> {
    pub fn new<C, V>(face: Face<'a>, chunk: &C) -> Self
    where
        C: Chunk<Output = V>,
        V: Voxel,
    {
        let aos = face_aos(&face, chunk);
        Self { face, aos }
    }

    pub fn aos(&self) -> [u32; 4] {
        self.aos
    }

    pub fn indices(&self, start: u32) -> [u32; 6] {
        let aos = self.aos();

        if (aos[1] + aos[2]) > (aos[0] + aos[3]) {
            [start, start + 2, start + 1, start + 1, start + 2, start + 3]
        } else {
            [start, start + 3, start + 1, start, start + 2, start + 3]
        }
    }
}

pub(crate) fn ao_value(side1: bool, corner: bool, side2: bool) -> u32 {
    match (side1, corner, side2) {
        (true, _, true) => 0,
        (true, true, false) | (false, true, true) => 1,
        (false, false, false) => 3,
        _ => 2,
    }
}

impl<'a> Deref for FaceWithAO<'a> {
    type Target = Face<'a>;

    fn deref(&self) -> &Self::Target {
        &self.face
    }
}

pub(crate) fn side_aos<V: Voxel>(neighbors: [V; 8]) -> [u32; 4] {
    let ns = [
        neighbors[0].visibility() == OPAQUE,
        neighbors[1].visibility() == OPAQUE,
        neighbors[2].visibility() == OPAQUE,
        neighbors[3].visibility() == OPAQUE,
        neighbors[4].visibility() == OPAQUE,
        neighbors[5].visibility() == OPAQUE,
        neighbors[6].visibility() == OPAQUE,
        neighbors[7].visibility() == OPAQUE,
    ];

    [
        ao_value(ns[0], ns[1], ns[2]),
        ao_value(ns[2], ns[3], ns[4]),
        ao_value(ns[6], ns[7], ns[0]),
        ao_value(ns[4], ns[5], ns[6]),
    ]
}

pub fn ao_to_color(ao: Vec<u32>) -> Vec<[f32; 4]> {
    let mut res = Vec::new();
    for value in ao {
        match value {
            0 => res.extend_from_slice(&[[0.1, 0.1, 0.1, 1.0]]),
            1 => res.extend_from_slice(&[[0.25, 0.25, 0.25, 1.0]]),
            2 => res.extend_from_slice(&[[0.5, 0.5, 0.5, 1.0]]),
            _ => res.extend_from_slice(&[[1., 1., 1., 1.0]]),
        }
    }
    res
}
