use crate::VoxValue;

/// One key/value pair of a [`VoxMap`](crate::VoxMap).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxMapEntry {
    /// The key.
    pub key: String,
    /// The value under the key.
    pub value: VoxValue,
}
