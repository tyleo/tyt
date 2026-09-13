use ty_math::{TyVector3F64, TyVector3I32};

/// The integer grid `origin`: the min corner offset from the placing node in
/// the node's local voxel frame. `round(box_min - center)` so the node
/// transform's position lands on the content center (the pivot); any odd-extent
/// half-voxel remainder is absorbed by that position, keeping rendering exact.
pub(crate) fn pivot_origin(box_min: [i32; 3], center: [f64; 3]) -> [i32; 3] {
    (TyVector3I32::from_array(box_min).as_dvec3() - TyVector3F64::from_array(center))
        .round()
        .as_ivec3()
        .to_array()
}
