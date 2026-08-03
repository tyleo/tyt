use crate::{srgba_u8_from_lin_srgba_f64, value_pool_lin_srgba_f64_color};
use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxValuePool};

/// Encodes the color at `value_id` in `value_pool` to sRGB `[r, g, b, a]`
/// bytes: [`value_pool_lin_srgba_f64_color`]'s read re-encoded for an 8-bit
/// consumer, `None` when that read is.
pub fn value_pool_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<[u8; 4]> {
    let color = value_pool_lin_srgba_f64_color(value_pool, value_id)?;
    Some(<[u8; 4]>::from(srgba_u8_from_lin_srgba_f64(color)))
}
