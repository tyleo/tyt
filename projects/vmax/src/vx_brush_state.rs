#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel Max brush-state tokens (`tools.bst`): the color- and gradient-mode
/// strings plus optional offset cursor indices. `ocx`/`ocn` are absent in some
/// files, so both are optional.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VXBrushState {
    /// Color mode (`cm`).
    pub cm: String,
    /// Color position (`cp`).
    pub cp: String,
    /// Gradient mode (`gm`).
    pub gm: String,
    /// Gradient position (`gp`).
    pub gp: String,
    /// Offset cursor max (`ocx`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ocx: Option<i64>,
    /// Offset cursor min (`ocn`).
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ocn: Option<i64>,
}
