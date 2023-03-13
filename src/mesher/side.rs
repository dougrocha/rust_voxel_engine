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
