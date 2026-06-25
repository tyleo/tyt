#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Voxel Max brush-state tokens (`tools.bst`): the color- and gradient-mode
/// strings plus optional offset cursor indices.
/// [`ocx`](Self::ocx)/[`ocn`](Self::ocn) are absent in some files, so both are
/// optional.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxBrushState {
    /// Color mode.
    pub cm: String,

    /// Color position.
    pub cp: String,

    /// Gradient mode.
    pub gm: String,

    /// Gradient position.
    pub gp: String,

    /// Offset cursor max.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ocx: Option<i64>,

    /// Offset cursor min.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub ocn: Option<i64>,

    /// Soft-falloff azimuth; present on some brush states.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sfaz: Option<f64>,

    /// Soft-falloff altitude; present on some brush states.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub sfat: Option<f64>,
}
