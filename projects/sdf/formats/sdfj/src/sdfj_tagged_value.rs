use crate::{SdfjIntValue, SdfjValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A property value that writes its kind: `{ "kind": "int", "value": ... }`
/// or `{ "kind": "json", "value": ... }`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(
    feature = "serde",
    serde(
        tag = "kind",
        content = "value",
        rename_all = "camelCase",
        deny_unknown_fields
    )
)]
pub enum SdfjTaggedValue {
    /// An `int` value.
    Int(SdfjIntValue),

    /// A `json` value.
    Json(SdfjValue),
}
