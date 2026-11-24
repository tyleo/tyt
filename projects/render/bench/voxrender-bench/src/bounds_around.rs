use ty_math::TyVector3F64;

/// The low and high corners of the box around `points`.
pub fn bounds_around(
    points: impl IntoIterator<Item = TyVector3F64>,
) -> (TyVector3F64, TyVector3F64) {
    let none = (
        TyVector3F64::splat(f64::INFINITY),
        TyVector3F64::splat(f64::NEG_INFINITY),
    );

    points
        .into_iter()
        .fold(none, |(low, high), point| (low.min(point), high.max(point)))
}
