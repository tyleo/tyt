use crate::BSdfMaterial;
use branded_id::U32Id;

/// The seam a `cells` pattern fills between its cells.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SdfBorder {
    /// The material filling the seam.
    pub material_id: U32Id<BSdfMaterial>,

    /// How wide the seam runs.
    pub width: f64,
}
