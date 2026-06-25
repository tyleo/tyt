/// One entry of a `.qbt` `COLORMAP`: an `RGBA` color, one byte per channel.
///
/// The color map is an optional palette, empty by default (the Qubicle
/// default), in which case voxels store their own `RGB`. When non-empty, a
/// voxel's red byte indexes this map (see [`QbtVoxel`](crate::qbt::QbtVoxel)).
/// Voxel bytes are stored verbatim and the index is not resolved, so files
/// round-trip either way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct QbtColor {
    /// Red channel.
    pub r: u8,

    /// Green channel.
    pub g: u8,

    /// Blue channel.
    pub b: u8,

    /// Alpha channel.
    pub a: u8,
}

impl QbtColor {
    /// A color from its four channels.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}
