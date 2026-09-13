use ty_math::TyVector3F64;

/// Grows the running `(min, max)` AABB to include the box centered at `center`
/// with half-extents `half`.
pub(crate) fn extend_bounds(
    bounds: &mut Option<([f64; 3], [f64; 3])>,
    center: [f64; 3],
    half: [f64; 3],
) {
    let center = TyVector3F64::from_array(center);
    let half = TyVector3F64::from_array(half);
    let lo = center - half;
    let hi = center + half;
    match bounds {
        Some((min, max)) => {
            *min = TyVector3F64::from_array(*min).min(lo).to_array();
            *max = TyVector3F64::from_array(*max).max(hi).to_array();
        }
        None => *bounds = Some((lo.to_array(), hi.to_array())),
    }
}
