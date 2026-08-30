#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One runtime object's editor-only state, which an editor preserves and the
/// runtime ignores.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VoxjEditObject {
    /// `[X, Y, Z]` size of the author's build volume in voxels.
    pub bounds: [u32; 3],

    /// `[X, Y, Z]` translation from the placing node to the build volume's min
    /// corner.
    pub origin: [i32; 3],
}
