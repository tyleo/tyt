/// How a ray-query candidate finds the first hit in its brick.
#[derive(Clone, Copy, Debug)]
pub enum BrickMarch {
    /// The tree walk over the brick along the placement's integer pixel ray.
    Integer,

    /// An f32 DDA over the brick along the hardware's ray.
    Float,
}

impl BrickMarch {
    /// The march's name in a report.
    pub fn label(self) -> &'static str {
        match self {
            BrickMarch::Integer => "integer",
            BrickMarch::Float => "float",
        }
    }

    /// The WGSL that defines `march`.
    pub fn wgsl(self) -> &'static str {
        match self {
            BrickMarch::Integer => include_str!("integer_march.wgsl"),
            BrickMarch::Float => include_str!("float_march.wgsl"),
        }
    }
}
