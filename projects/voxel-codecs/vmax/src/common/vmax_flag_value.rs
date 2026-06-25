#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Value carried by a [`VMaxFlag`](crate::VMaxFlag) (`x`). Serializes untagged
/// as a bare `bool` or integer.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VMaxFlagValue {
    /// A boolean flag (`tools.mr`, `tools.st`).
    Bool(bool),

    /// An integer flag (`tools.stf`).
    Int(i64),
}

impl Default for VMaxFlagValue {
    /// [`Bool(false)`](Self::Bool); overwritten on decode, never read directly.
    fn default() -> Self {
        VMaxFlagValue::Bool(false)
    }
}
