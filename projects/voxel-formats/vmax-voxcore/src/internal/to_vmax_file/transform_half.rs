use ty_math::{TyTransformF64, TyVector3F64};

/// The half-extent of the AABB of a box rotated and scaled by a node transform.
/// A box centered on its pivot stays centered under the transform, so only the
/// half-extent picks up the rotation: `sum_j abs(col_j) * half[j]` over the
/// rotated, scaled basis columns.
pub(crate) fn transform_half(transform: &TyTransformF64, half: [f64; 3]) -> [f64; 3] {
    let col_x = transform.rotation * TyVector3F64::new(transform.scale.x, 0.0, 0.0);
    let col_y = transform.rotation * TyVector3F64::new(0.0, transform.scale.y, 0.0);
    let col_z = transform.rotation * TyVector3F64::new(0.0, 0.0, transform.scale.z);
    [
        col_x.x.abs() * half[0] + col_y.x.abs() * half[1] + col_z.x.abs() * half[2],
        col_x.y.abs() * half[0] + col_y.y.abs() * half[1] + col_z.y.abs() * half[2],
        col_x.z.abs() * half[0] + col_y.z.abs() * half[1] + col_z.z.abs() * half[2],
    ]
}
