/// The axis-aligned bounding box of a triangle soup as `(min, max)`, or `None`
/// when there are no triangles.
pub(crate) fn triangle_bounds(triangles: &[[[f64; 3]; 3]]) -> Option<([f64; 3], [f64; 3])> {
    let mut points = triangles.iter().flatten().copied();
    let first = points.next()?;
    let (mut min, mut max) = (first, first);
    for [x, y, z] in points {
        min = [min[0].min(x), min[1].min(y), min[2].min(z)];
        max = [max[0].max(x), max[1].max(y), max[2].max(z)];
    }
    Some((min, max))
}
