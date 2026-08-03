use branded_id::U32Id;
use ty_math::TyLinSrgbaF64;
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// Reads the color at `value_id` in `value_pool` as stored: linear light, as
/// a [`TyLinSrgbaF64`], or `None` when the value id is not the value pool's
/// or `value_pool` holds no float vectors. The kind does not say the values
/// are colors. The caller does, from the property name it resolved. A
/// three-component value takes opaque alpha.
pub(crate) fn value_pool_lin_srgba_f64_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<TyLinSrgbaF64> {
    match value_pool.value(value_id)? {
        VoxValuePoolValueRef::Vec3Float(&[red, green, blue]) => {
            Some(TyLinSrgbaF64::new(red, green, blue, 1.0))
        }
        VoxValuePoolValueRef::Vec4Float(&[red, green, blue, alpha]) => {
            Some(TyLinSrgbaF64::new(red, green, blue, alpha))
        }
        _ => None,
    }
}
