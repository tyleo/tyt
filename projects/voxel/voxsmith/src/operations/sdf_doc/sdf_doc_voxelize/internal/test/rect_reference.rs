use ty_math::TyVector2F64;

/// The signed distance from `point` to the rectangle between `min` and `max`:
/// to the clamped point outside and to the nearest side inside.
pub fn rect_reference(point: TyVector2F64, min: TyVector2F64, max: TyVector2F64) -> f64 {
    let clamped = point.clamp(min, max);

    if clamped != point {
        return point.distance(clamped);
    }

    -(point - min).min(max - point).min_element()
}
