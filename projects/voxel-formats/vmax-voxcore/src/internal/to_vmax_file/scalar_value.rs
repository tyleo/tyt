use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// The `f64` at `value_id` in a `float` value pool, or `None`.
pub(crate) fn scalar_value(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<f64> {
    match value_pool.value(value_id) {
        Some(VoxValuePoolValueRef::Float(number)) => Some(number),
        _ => None,
    }
}
