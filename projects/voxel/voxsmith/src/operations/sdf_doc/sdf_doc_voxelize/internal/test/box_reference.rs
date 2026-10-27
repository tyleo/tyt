use ty_math::TyVector3F64;

/// The signed distance from `point` to the box between `min` and `max`: to the
/// clamped point outside and to the nearest face inside.
pub fn box_reference(point: TyVector3F64, min: TyVector3F64, max: TyVector3F64) -> f64 {
    let clamped = point.clamp(min, max);

    if clamped != point {
        return point.distance(clamped);
    }

    -(point - min).min(max - point).min_element()
}
