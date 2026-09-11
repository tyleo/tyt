use crate::VoxjValue;

/// One key/value pair of a [`VoxjMap`](crate::VoxjMap).
#[derive(Clone, Debug, PartialEq)]
pub struct VoxjMapEntry {
    /// The key.
    pub key: String,
    /// The value under the key.
    pub value: VoxjValue,
}
