/// The parsed color table of a `palette*.png` file: one `[r, g, b, a]` cell per
/// pixel, in image order. Voxel Max stores a palette's colors as a `256x1` RGBA
/// strip; this is the decoded form of that image.
///
/// It is encoded to and from PNG bytes rather than through serde, so it carries
/// no serde derives.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxPalettePngFile(pub Vec<[u8; 4]>);
