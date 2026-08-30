/// One cell of a [`GoxlBlock`](crate::GoxlBlock) grid: an `RGBA` color whose
/// alpha channel doubles as presence. Stored on disk as one pixel of the block's
/// `BL16` PNG.
///
/// [`a`](Self::a)` == 0` is an empty cell; a non-zero alpha is a solid voxel
/// (Goxel writes `255`). The `RGB` channels carry the voxel's color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct GoxlVoxel {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Alpha channel, doubling as presence: `0` is an empty cell, non-zero a
    /// solid voxel.
    pub a: u8,
}

impl GoxlVoxel {
    /// A solid voxel of the given color, with alpha `255`.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Whether the cell is empty (its [`a`](Self::a) channel is `0`).
    pub const fn is_empty(self) -> bool {
        self.a == 0
    }
}
