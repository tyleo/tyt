use ty_math::TyVector2F64;

/// The distance from `point` to the segment from `a` to `b`.
pub fn segment_distance2d(point: TyVector2F64, a: TyVector2F64, b: TyVector2F64) -> f64 {
    let t = ((point - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
    point.distance(a.lerp(b, t))
}
