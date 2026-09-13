use branded_id::U32Id;
use voxcore::{BVoxValuePoolValue, VoxValuePool, color::value_pool_lin_srgba_f64_color};

/// The linear rgb at `value_id` in a color value pool, or `None` when the pool
/// holds no float vectors.
pub(crate) fn pool_color(
    value_pool: &VoxValuePool,
    value_id: U32Id<BVoxValuePoolValue>,
) -> Option<[f64; 3]> {
    let color = value_pool_lin_srgba_f64_color(value_pool, value_id)?;
    Some([color.red, color.green, color.blue])
}
