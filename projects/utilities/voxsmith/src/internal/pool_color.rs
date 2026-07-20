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
        VoxValuePool::Srgb { values } => values
            .get(value_id)
            .map(|&[r, g, b]| TySrgbaF64::new(r, g, b, 1.0).to_u8().to_array()),
        VoxValuePool::Srgba { values } => values
            .get(value_id)
            .map(|&[r, g, b, a]| TySrgbaF64::new(r, g, b, a).to_u8().to_array()),
        VoxValuePool::LinearRgb { values } => values.get(value_id).map(|&[r, g, b]| {
            TyLinSrgbaF64::new(r, g, b, 1.0)
                .to_srgba()
                .to_u8()
                .to_array()
        }),
        VoxValuePool::LinearRgba { values } => values
            .get(value_id)
            .map(|&[r, g, b, a]| TyLinSrgbaF64::new(r, g, b, a).to_srgba().to_u8().to_array()),
        _ => None,
    }
}
