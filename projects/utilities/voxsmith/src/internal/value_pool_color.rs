use branded_id::U32Id;
use ty_math::{TyLinSrgbaF64, TySrgbaF64};
use voxcore::{BVoxValuePoolValue, VoxValuePool, VoxValuePoolValueRef};

/// Decodes the color at `value_id` in a color `value_pool` to sRGB
/// `[r, g, b, a]` bytes, or `None` when the value id is not the value pool's or
/// `value_pool` is not a color kind. An sRGB value pool's components map
/// straight to bytes, and a linear value pool's re-encode to sRGB. A
/// three-component color takes opaque alpha.
pub fn value_pool_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<[u8; 4]> {
    match value_pool.value(value_id)? {
        VoxValuePoolValueRef::Srgb(&[r, g, b]) => Some(<[u8; 4]>::from(
            TySrgbaF64::new(r, g, b, 1.0).into_format::<u8, u8>(),
        )),
        VoxValuePoolValueRef::Srgba(&[r, g, b, a]) => Some(<[u8; 4]>::from(
            TySrgbaF64::new(r, g, b, a).into_format::<u8, u8>(),
        )),
        VoxValuePoolValueRef::LinearRgb(&[r, g, b]) => Some(<[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, 1.0)).into_format::<u8, u8>(),
        )),
        VoxValuePoolValueRef::LinearRgba(&[r, g, b, a]) => Some(<[u8; 4]>::from(
            TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>(),
        )),
        _ => None,
    }
}
