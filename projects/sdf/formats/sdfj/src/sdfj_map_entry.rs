use crate::SdfjValue;

/// One key/value pair of an [`SdfjMap`](crate::SdfjMap).
#[derive(Clone, Debug, PartialEq)]
pub struct SdfjMapEntry<TValue = SdfjValue> {
    /// The key.
    pub key: String,

    /// The value under the key.
    pub value: TValue,
}
