use voxcore::{VoxExt, VoxValue};

/// One entry of a Voxel Json document's `ext` block kept as it was parsed,
/// under a key no enabled format owns. It follows no hook, so it goes stale
/// under a mutation that moves a listing its value indexes.
#[derive(Clone, Debug, PartialEq)]
pub struct InertVoxExt {
    /// The key the entry sits under.
    pub key: String,

    /// The entry's value.
    pub value: VoxValue,
}

impl VoxExt for InertVoxExt {}
