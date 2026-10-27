use crate::SdfValue;

/// One key/value pair of an [`SdfMap`](crate::SdfMap).
#[derive(Clone, Debug, PartialEq)]
pub struct SdfMapEntry {
    /// The key.
    pub key: String,

    /// The value under the key.
    pub value: SdfValue,
}
