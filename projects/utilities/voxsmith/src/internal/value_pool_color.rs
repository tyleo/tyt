use crate::srgba_u8_from_linear_color;
use branded_id::U32Id;
use ty_math::TyLinSrgbaF64;
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// Encodes the color at `value_id` in `value_pool` to sRGB `[r, g, b, a]`
/// bytes, or `None` when the value id is not the value pool's or `value_pool`
/// holds no float vectors. The kind does not say the values are colors; the
/// caller does, from the property name it resolved. Values are linear light
/// per the format, re-encoded to sRGB here; a three-component value takes
/// opaque alpha.
pub fn value_pool_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<[u8; 4]> {
    let color = match value_pool.value(value_id)? {
        VoxValuePoolValueRef::Vec3Float(&[red, green, blue]) => {
            TyLinSrgbaF64::new(red, green, blue, 1.0)
        }
        VoxValuePoolValueRef::Vec4Float(&[red, green, blue, alpha]) => {
            TyLinSrgbaF64::new(red, green, blue, alpha)
        }
        _ => return None,
    };
    Some(<[u8; 4]>::from(srgba_u8_from_linear_color(color)))
}
