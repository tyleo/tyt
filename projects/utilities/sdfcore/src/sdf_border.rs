use crate::BSdfMaterial;
use branded_id::U32Id;

/// The seam a `bricks` or `cells` pattern fills between its pieces.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfBorder {
    /// The material filling the seam.
    pub material_id: U32Id<BSdfMaterial>,

    /// How wide the seam runs.
    pub width: f64,
}
