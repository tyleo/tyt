#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel Max view/edit partition box (`tools.vp`): the inclusive `[x, y, z]`
/// voxel bounds the editor's tools are scoped to.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxViewBox {
    /// Minimum `[x, y, z]` corner.
    pub min: [i64; 3],

    /// Maximum `[x, y, z]` corner.
    pub max: [i64; 3],
}
