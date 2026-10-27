use ty_math::TyVector2F64;

/// The distance from `point` to the circle of `radius` about `center` from
/// `from_degrees` counterclockwise to `to_degrees`.
pub fn arc_curve_distance(
    point: TyVector2F64,
    center: TyVector2F64,
    radius: f64,
    from_degrees: f64,
    to_degrees: f64,
) -> f64 {
    let offset = point - center;
    let angle = offset.y.atan2(offset.x).to_degrees();
    let past_from = (angle - from_degrees).rem_euclid(360.0);

    if past_from <= to_degrees - from_degrees {
        return (offset.length() - radius).abs();
    }

    let end = |degrees: f64| {
        center + TyVector2F64::new(degrees.to_radians().cos(), degrees.to_radians().sin()) * radius
    };

    point
        .distance(end(from_degrees))
        .min(point.distance(end(to_degrees)))
}
