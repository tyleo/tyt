#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The parsed color table of a `palette*.png` file: one `[r, g, b, a]` cell per
/// pixel, in image order. Voxel Max stores a palette's colors as a `256x1` RGBA
/// strip; this is the decoded form of that image.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxPalettePngFile(pub Vec<[u8; 4]>);
