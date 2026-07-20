use branded_id::U32Id;
use voxcore::{BVoxArrayProperty, BVoxLayer, BVoxPalette, BVoxScalarProperty};

/// One property's winning supplier for a whole object. Under the canonical
/// override order the last layer that supplies the property wins; a layer
/// always supplies its palette's scalar properties, and its array
/// properties only while sampled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectPropertyRef {
    /// One value per voxel: the property reads the material the voxel samples
    /// in the winning layer.
    Array {
        /// The winning sampled layer.
        layer: U32Id<BVoxLayer>,

        /// The palette the winning layer references.
        palette: U32Id<BVoxPalette>,

        /// The array property supplying the value.
        property: U32Id<BVoxArrayProperty>,
    },

    /// One value for the whole object, pinned by the winning palette.
    Scalar {
        /// The palette the winning layer references.
        palette: U32Id<BVoxPalette>,

        /// The scalar property pinning the value.
        property: U32Id<BVoxScalarProperty>,
    },
}
