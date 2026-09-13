/// How a texture coordinate outside `0..1` maps back onto the image.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum MeshWrap {
    /// Tile the image.
    #[default]
    Repeat,

    /// Clamp to the nearest edge texel.
    ClampToEdge,

    /// Tile the image, flipping every other repeat.
    MirroredRepeat,
}
