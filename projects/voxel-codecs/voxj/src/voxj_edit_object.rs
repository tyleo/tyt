#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One runtime object's edit grid: the author's build volume, which contains
/// the runtime grid. Held in
/// [`VoxjEditState::objects`](crate::VoxjEditState::objects), aligned by index
/// with the runtime objects.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjEditObject {
    /// `[X, Y, Z]` size of the edit grid in voxels.
    pub bounds: [u32; 3],

    /// `[X, Y, Z]` translation from the placing node to the edit grid's min
    /// corner.
    pub origin: [i32; 3],
}
