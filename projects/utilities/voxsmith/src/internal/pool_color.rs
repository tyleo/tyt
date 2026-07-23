use branded_id::U32Id;
use ty_math::{TyLinSrgbaF64, TySrgbaF64};
use voxcore::{BVoxPoolValue, VoxValuePool};

/// Decodes the color at `value_id` in a color `pool` to sRGB `[r, g, b, a]`
/// bytes, or `None` when the value id is out of range or `pool` is not a color
/// kind. An sRGB pool's components map straight to bytes; a linear pool's
/// re-encode to sRGB. A three-component color takes opaque alpha.
pub fn pool_color(pool: &VoxValuePool, value_id: U32Id<BVoxPoolValue>) -> Option<[u8; 4]> {
    let value_id = value_id.to_usize_id();

    match pool {
        VoxValuePool::Srgb { values } => values.get(value_id).map(|&[r, g, b]| {
            <[u8; 4]>::from(TySrgbaF64::new(r, g, b, 1.0).into_format::<u8, u8>())
        }),
        VoxValuePool::Srgba { values } => values.get(value_id).map(|&[r, g, b, a]| {
            <[u8; 4]>::from(TySrgbaF64::new(r, g, b, a).into_format::<u8, u8>())
        }),
        VoxValuePool::LinearRgb { values } => values.get(value_id).map(|&[r, g, b]| {
            <[u8; 4]>::from(
                TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, 1.0)).into_format::<u8, u8>(),
            )
        }),
        VoxValuePool::LinearRgba { values } => values.get(value_id).map(|&[r, g, b, a]| {
            <[u8; 4]>::from(
                TySrgbaF64::from_linear(TyLinSrgbaF64::new(r, g, b, a)).into_format::<u8, u8>(),
            )
        }),
        _ => None,
    }
}
