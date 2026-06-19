use serde::{Deserialize, Serialize};

/// The color payload of a Voxel Max brush slot (`brush.brushes[].c`): the `_dm`
/// triple Voxel Max stores per slot.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXBrushColorSerde {
    /// The `_dm` triple (observed as `[1, 1, 1]`).
    #[serde(rename = "_dm")]
    pub dm: Vec<i64>,
}
