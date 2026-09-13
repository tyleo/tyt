use crate::Result;
use branded_id::U32Id;
use voxcore::{BVoxValuePool, VoxMain, VoxValuePool};

/// A float value pool over `values`, defaulting a NaN coefficient to zero so
/// the value pool builds. The infinities the wire spells carry across, and the
/// exact value rides in the ext. Errors when `values` is empty.
pub(crate) fn float_value_pool(
    main: &mut VoxMain<()>,
    values: Vec<f64>,
) -> Result<U32Id<BVoxValuePool>> {
    let values = values
        .into_iter()
        .map(|v| if v.is_nan() { 0.0 } else { v })
        .collect();
    Ok(main.retain_value_pool(VoxValuePool::float(values)?))
}
