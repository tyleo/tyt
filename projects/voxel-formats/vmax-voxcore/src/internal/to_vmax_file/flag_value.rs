use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxEffectiveProperty, VoxValuePoolValueRef};

/// The `bool` at `value_id` in a property's `bool` value pool, or `None`.
pub(crate) fn flag_value(
    property: &VoxEffectiveProperty,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<bool> {
    match property.value_pool().value(value_id) {
        Some(VoxValuePoolValueRef::Bool(flag)) => Some(flag),
        _ => None,
    }
}
