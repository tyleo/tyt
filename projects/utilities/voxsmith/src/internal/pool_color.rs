use branded_id::U32Id;
use ty_math::{TyLinSrgbaF64, TySrgbaF64};
use voxcore::{BVoxPoolValue, VoxPoolValueRef, VoxValuePool};

/// Decodes the color at `value_id` in a color `pool` to sRGB `[r, g, b, a]`
/// bytes, or `None` when the value id is not the pool's or `pool` is not a
/// color kind. An sRGB pool's components map straight to bytes, and a linear
/// pool's re-encode to sRGB. A three-component color takes opaque alpha.
pub fn pool_color(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> Option<[u8; 4]> {
    match pool.value(value_id)? {
        VoxPoolValueRef::Srgb(&[r, g, b]) => Some(<[u8; 4]>::from(
            TySrgbaF64::new(r, g, b, 1.0).into_format::<u8, u8>(),
        )),
        VoxPoolValueRef::Srgba(&[r, g, b, a]) => Some(<[u8; 4]>::from(
            TySrgbaF64::new(r, g, b, a).into_format::<u8, u8>(),
        )),
        VoxPoolValueRef::LinearRgb(&[r, g, b]) => Some(<[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, 1.0)).into_format::<u8, u8>(),
        )),
        VoxPoolValueRef::LinearRgba(&[r, g, b, a]) => Some(<[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>(),
        )),
        _ => None,
    }
}
