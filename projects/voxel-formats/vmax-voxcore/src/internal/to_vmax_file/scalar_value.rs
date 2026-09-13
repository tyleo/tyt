use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxEffectiveProperty, VoxValuePoolValueRef};

/// The `f64` at `value_id` in a property's `float` value pool, or `None`.
pub(crate) fn scalar_value(
    property: &VoxEffectiveProperty,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<f64> {
    match property.value_pool().value(value_id) {
        Some(VoxValuePoolValueRef::Float(number)) => Some(number),
        _ => None,
    }
}
