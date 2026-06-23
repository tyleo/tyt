#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The color payload of a Voxel Max brush slot (`brush.brushes[].c`).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxBrushColor {
    /// The triple (observed as `[1, 1, 1]`).
    #[cfg_attr(feature = "serde", serde(rename = "_dm"))]
    pub dm: Vec<i64>,
}
