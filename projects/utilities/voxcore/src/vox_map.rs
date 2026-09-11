use crate::{VoxMapEntry, VoxValue};

/// An ordered set of key/value pairs: the object form of a [`VoxValue`].
///
/// Insertion order is preserved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxMap(Vec<VoxMapEntry>);

impl VoxMap {
    /// A map of `entries`, in their order.
    pub fn new(entries: Vec<VoxMapEntry>) -> Self {
        Self(entries)
    }

    /// The value under `key`, or `None` when no entry has it.
    pub fn get(&self, key: &str) -> Option<&VoxValue> {
        self.0
            .iter()
            .find(|entry| entry.key == key)
            .map(|entry| &entry.value)
    }

    /// The entries, in insertion order.
    pub fn entries(&self) -> &[VoxMapEntry] {
        &self.0
    }

    /// The entries, taken out of the map.
    pub fn into_entries(self) -> Vec<VoxMapEntry> {
        self.0
    }
}
