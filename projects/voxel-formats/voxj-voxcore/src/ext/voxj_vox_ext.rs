use voxcore::{VoxExt, VoxMapEntry, VoxValue};

/// A document's `ext` block kept as it was parsed, one slot per key. An
/// empty block stands for a document with no block. The block is not
/// understood, so it follows no hook and goes stale under a mutation that
/// moves a listing. A crate that knows the block's entries decodes them into
/// an ext that follows.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoxjVoxExt(Vec<VoxMapEntry>);

impl VoxjVoxExt {
    /// An ext keeping `slots` as the block, in their order.
    pub fn new(slots: Vec<VoxMapEntry>) -> Self {
        Self(slots)
    }

    /// The slot under `key`, or `None` when the block has no such key.
    pub fn slot(&self, key: &str) -> Option<&VoxValue> {
        self.0
            .iter()
            .find(|slot| slot.key == key)
            .map(|slot| &slot.value)
    }

    /// The slots, in the block's order.
    pub fn slots(&self) -> &[VoxMapEntry] {
        &self.0
    }

    /// The slots, taken out of the ext.
    pub fn into_slots(self) -> Vec<VoxMapEntry> {
        self.0
    }

    /// Whether the block has no slots.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl VoxExt for VoxjVoxExt {}
