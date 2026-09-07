use ty_math::TyVector3F64;

/// The axis-aligned bounding box of a soup of points as `(min, max)`, or `None`
/// when the iterator is empty.
pub(crate) fn triangle_bounds(
    points: impl IntoIterator<Item = TyVector3F64>,
) -> Option<(TyVector3F64, TyVector3F64)> {
    let mut points = points.into_iter();

    let first = points.next()?;

    let (mut min, mut max) = (first, first);

    for p in points {
        min = min.min(p);
        max = max.max(p);
    }

    Some((min, max))
}
