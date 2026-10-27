use ty_math::TyVector3F64;

/// The distance from `point` to the segment from `a` to `b`.
pub fn segment_distance3d(point: TyVector3F64, a: TyVector3F64, b: TyVector3F64) -> f64 {
    let t = ((point - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
    point.distance(a.lerp(b, t))
}
