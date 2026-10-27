use crate::SdfjTaggedValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A material property's value. A boolean, a number, a string, or a list of
/// numbers writes as itself, and an `int` or a `json` value writes tagged.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum SdfjPropertyValue {
    /// A boolean.
    Bool(bool),

    /// A number.
    Number(#[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))] f64),

    /// A list of numbers.
    NumberArray(#[cfg_attr(feature = "serde", serde(serialize_with = "crate::finite"))] Vec<f64>),

    /// An `int` or a `json` value.
    Tagged(SdfjTaggedValue),

    /// A string.
    Text(String),
}
