/// Parsed color table of a `palette*.png` file: one `[r, g, b, a]` cell per
/// pixel, in image order. Voxel Max stores a palette's colors as a `256x1` RGBA
/// strip. Encoded to and from PNG, not serde.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxPalettePngFile(pub Vec<[u8; 4]>);
