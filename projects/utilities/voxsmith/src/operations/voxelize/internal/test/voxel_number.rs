use crate::operations::voxelize::voxel_attribute;
use ty_math::TyVector3U32;
use voxcore::{VoxMain, VoxValuePoolValueRef};

/// The numeric value of one float attribute the voxel at `position` samples.
pub(crate) fn voxel_number(main: &VoxMain, position: TyVector3U32, attribute: &str) -> f64 {
    let (value_pool, value_id) = voxel_attribute(main, position, attribute);
    match value_pool.value(value_id).unwrap() {
        VoxValuePoolValueRef::Float(number) => number,
        other => panic!("{attribute} is a float, not {other:?}"),
    }
}
