use ty_math::TyVector3F32;

/// A box of another placement that shares space with a box, as the box's
/// fragments read it from `mask_march.wgsl`'s `crossers`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoxCrosser {
    /// The other box's index among the scene's marched boxes followed by
    /// its solid ones, plus [`SOLID`](Self::SOLID) for a solid one.
    pub entry: u32,

    /// The low and high corners of a box around the space the two share, in
    /// the cells of the first box's tile.
    pub low: TyVector3F32,
    pub high: TyVector3F32,
}

impl BoxCrosser {
    /// The bit of an entry that marks a solid box.
    pub const SOLID: u32 = 1 << 31;

    /// The crosser's two `vec4<f32>`s as their bits: its low corner and its
    /// entry, then its high corner and a zero.
    pub fn words(&self) -> [u32; 8] {
        let [low_x, low_y, low_z] = self.low.to_array().map(f32::to_bits);
        let [high_x, high_y, high_z] = self.high.to_array().map(f32::to_bits);

        [low_x, low_y, low_z, self.entry, high_x, high_y, high_z, 0]
    }
}
