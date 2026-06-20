#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The value carried by a [`VXFlag`](crate::VXFlag) (`x`): a boolean or integer
/// depending on the flag. Serializes untagged as a bare `bool` or integer, so a
/// flag round-trips either kind without coercion.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum VXFlagValue {
    /// A boolean flag (`tools.mr`, `tools.st`).
    Bool(bool),
    /// An integer flag (`tools.stf`).
    Int(i64),
}

impl Default for VXFlagValue {
    /// Defaults to [`Bool(false)`](Self::Bool); a flag's value is overwritten
    /// on decode and the field is never read directly.
    fn default() -> Self {
        VXFlagValue::Bool(false)
    }
}
