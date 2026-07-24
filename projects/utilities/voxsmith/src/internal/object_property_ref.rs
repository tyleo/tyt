use branded_id::U32Id;
use voxcore::{BVoxArrayProperty, BVoxLayer, BVoxPalette};

/// One property's winning supplier for a whole object. Under the canonical
/// override order the last layer whose palette carries the property wins;
/// the property reads the material each voxel samples in that layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectPropertyRef {
    /// The winning layer.
    pub layer: U32Id<BVoxLayer>,

    /// The palette the winning layer references.
    pub palette: U32Id<BVoxPalette>,

    /// The array property supplying the value.
    pub property: U32Id<BVoxArrayProperty>,
}
