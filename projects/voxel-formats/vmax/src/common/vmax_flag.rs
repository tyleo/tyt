use crate::VMaxFlagValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max tool flag (`{x: ...}`), used by `tools.mr` / `tools.st`
/// (boolean), `tools.stf` (integer), and the history edit command's
/// `mir`/`str`/`strf`.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxFlag {
    /// Flag value, or the x axis on a per-axis flag.
    pub x: VMaxFlagValue,

    /// Y axis on a per-axis flag.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub y: Option<VMaxFlagValue>,

    /// Z axis on a per-axis flag.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub z: Option<VMaxFlagValue>,
}
