/// One cell of a [`QbtMatrix`](crate::qbt::QbtMatrix) grid: an `RGB` color and a
/// visibility mask. Stored on disk as four bytes (`r`, `g`, `b`, `mask`).
///
/// [`mask`](Self::mask)` == 0` is an empty cell; non-zero marks a solid voxel
/// and is a per-face visible-sides bitmask.
///
/// When the file's [`color_map`](crate::qbt::QbtFile::color_map) is non-empty,
/// [`r`](Self::r) is a palette index rather than a red channel. The bytes are
/// stored as read either way; the index is not resolved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct QbtVoxel {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Visibility mask: `0` for an empty cell, otherwise a per-face bitmask of
    /// the visible sides.
    pub mask: u8,
}

impl QbtVoxel {
    /// A voxel from its color channels and visibility mask.
    pub const fn new(r: u8, g: u8, b: u8, mask: u8) -> Self {
        Self { r, g, b, mask }
    }

    /// Whether the cell is empty (its [`mask`](Self::mask) is `0`).
    pub const fn is_empty(self) -> bool {
        self.mask == 0
    }
}
