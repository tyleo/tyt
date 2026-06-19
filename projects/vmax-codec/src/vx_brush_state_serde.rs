use serde::{Deserialize, Serialize};

/// Voxel Max brush-state tokens (`tools.bst`): the color- and gradient-mode
/// strings plus optional offset cursor indices. `ocx`/`ocn` are absent in some
/// files, so both are optional.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct VXBrushStateSerde {
    /// Color mode (`cm`).
    pub cm: String,
    /// Color position (`cp`).
    pub cp: String,
    /// Gradient mode (`gm`).
    pub gm: String,
    /// Gradient position (`gp`).
    pub gp: String,
    /// Offset cursor max (`ocx`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocx: Option<i64>,
    /// Offset cursor min (`ocn`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ocn: Option<i64>,
}
