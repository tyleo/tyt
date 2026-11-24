use bytemuck::{Pod, Zeroable};

/// A box of another placement that shares space with a box, as the box's
/// fragments read it from `mask_march.wgsl`'s `crossers`: two `vec4<f32>`s.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BoxCrosser {
    /// The low corner of a box around the space the two share, in the cells
    /// of the first box's tile.
    pub low: [f32; 3],

    /// The other box's index among the scene's marched boxes followed by
    /// its solid ones, plus [`SOLID`](Self::SOLID) for a solid one.
    pub entry: u32,

    /// The high corner of the box around the shared space.
    pub high: [f32; 3],

    pub pad: u32,
}

impl BoxCrosser {
    /// The bit of an entry that marks a solid box.
    pub const SOLID: u32 = 1 << 31;
}
