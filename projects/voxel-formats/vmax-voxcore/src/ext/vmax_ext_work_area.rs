#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A work area that hides voxels, in the object's node-local voxel frame on
/// Voxel Max's Z-up axes.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct VMaxExtWorkArea {
    /// Min corner.
    pub min: [i32; 3],

    /// Max corner, inclusive.
    pub max: [i32; 3],
}
