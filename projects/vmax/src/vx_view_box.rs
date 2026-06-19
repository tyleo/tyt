use serde::{Deserialize, Serialize};

/// The Voxel Max view/edit partition box (`tools.vp`): the inclusive `[x, y, z]`
/// min and max voxel bounds the editor's tools are scoped to.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXViewBox {
    /// Minimum `[x, y, z]` corner.
    pub min: [i64; 3],
    /// Maximum `[x, y, z]` corner.
    pub max: [i64; 3],
}
