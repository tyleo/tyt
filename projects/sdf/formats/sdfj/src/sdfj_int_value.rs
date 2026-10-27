#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// What an `int` value holds: one number or a list of numbers.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SdfjIntValue {
    /// One number.
    Number(#[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))] f64),

    /// A list of numbers.
    NumberArray(#[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))] Vec<f64>),
}
