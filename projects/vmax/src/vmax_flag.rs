use crate::VMaxFlagValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A single-value Voxel Max tool flag (`{x: ...}`), used by `tools.mr`,
/// `tools.st` (boolean) and `tools.stf` (integer). The payload is kept as a
/// generic [`VMaxFlagValue`] so the one struct round-trips either kind without
/// coercion.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct VMaxFlag {
    /// The flag value, a boolean or integer depending on the flag.
    pub x: VMaxFlagValue,
}
