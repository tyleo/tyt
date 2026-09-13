use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// The `bool` at `value_id` in a `bool` value pool, or `None`.
pub(crate) fn flag_value(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<bool> {
    match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Bool(flag)) => Some(flag),
        _ => None,
    }
}
