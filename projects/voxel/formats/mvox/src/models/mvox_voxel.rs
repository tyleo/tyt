/// One solid voxel: integer grid coordinates and a palette color index. Stored
/// in an `XYZI` chunk as four bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MVoxVoxel {
    /// X coordinate, `0..size[0]`.
    pub x: u8,

    /// Y coordinate, `0..size[1]`.
    pub y: u8,

    /// Z coordinate, `0..size[2]` (gravity runs along Z).
    pub z: u8,

    /// Palette color index in `1..=255` (`0` is the empty slot); selects
    /// [`MVoxPalette::colors`](crate::MVoxPalette::colors)`[color_index]`.
    pub color_index: u8,
}
