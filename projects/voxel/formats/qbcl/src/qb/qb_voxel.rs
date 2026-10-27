/// One cell of a [`QbMatrix`](crate::qb::QbMatrix) grid: a logical `RGB` color
/// and a visibility byte. Stored on disk as four bytes.
///
/// The fourth byte is the `.qb` visibility mask, not an alpha:
/// [`visibility`](Self::visibility)` == 0` is an empty cell, non-zero a solid
/// voxel. When the file's
/// [`visibility_mask_encoded`](crate::qb::QbFile::visibility_mask_encoded) flag
/// is set, that value is a per-face visible-sides bitmask; otherwise it is
/// `255`. Color channels are normalized to `RGB`, undoing a file's `BGRA`
/// [`color_format`](crate::qb::QbFile::color_format).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct QbVoxel {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Visibility byte: `0` for an empty cell, non-zero for a solid voxel.
    pub visibility: u8,
}

impl QbVoxel {
    /// A solid voxel of the given color, with visibility `255`.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b,
            visibility: 255,
        }
    }

    /// Whether the cell is empty (its [`visibility`](Self::visibility) is `0`).
    pub const fn is_empty(self) -> bool {
        self.visibility == 0
    }
}
