use crate::SdfMapEntry;

/// An ordered set of key/value pairs: the object form of an
/// [`SdfValue`](crate::SdfValue).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SdfMap(Vec<SdfMapEntry>);

impl SdfMap {
    /// A map of `entries`, in their order.
    pub fn new(entries: Vec<SdfMapEntry>) -> Self {
        Self(entries)
    }

    /// The entries, in insertion order.
    pub fn entries(&self) -> &[SdfMapEntry] {
        &self.0
    }

    /// The entries, taken out of the map.
    pub fn into_entries(self) -> Vec<SdfMapEntry> {
        self.0
    }
}
