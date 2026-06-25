use crate::VMaxFlagValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A Voxel Max tool flag (`{x: ...}`), used by `tools.mr` / `tools.st`
/// (boolean), `tools.stf` (integer), and the history edit command's
/// `mir`/`str`/`strf`. The payload is kept as a generic [`VMaxFlagValue`] so the
/// one struct round-trips either kind without coercion. Per-axis flags (the
/// mirror, for instance) also carry [`y`](Self::y)/[`z`](Self::z); both are
/// absent on single-value flags.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default, deny_unknown_fields))]
pub struct VMaxFlag {
    /// The flag value (or the x axis), a boolean or integer.
    pub x: VMaxFlagValue,

    /// The y-axis value on a per-axis flag; absent otherwise.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub y: Option<VMaxFlagValue>,

    /// The z-axis value on a per-axis flag; absent otherwise.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub z: Option<VMaxFlagValue>,
}
