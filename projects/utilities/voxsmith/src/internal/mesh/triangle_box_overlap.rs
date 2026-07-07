use ty_math::TyVector3F64;

/// Whether a triangle overlaps an axis-aligned cube, by the separating-axis
/// theorem: the two are disjoint exactly when some axis separates their
/// projections. The 13 candidate axes are the 3 cube face normals, the triangle
/// normal, and the 9 cross products of a cube edge with a triangle edge.
///
/// # Arguments
/// * `center` - the cube's center.
/// * `half` - the cube's half-edge length on each axis.
/// * `triangle` - the triangle's three vertices.
pub(crate) fn triangle_box_overlap(center: [f64; 3], half: f64, triangle: &[[f64; 3]; 3]) -> bool {
    let center = TyVector3F64::from_array(center);
    // The triangle relative to the cube center, so the cube is centered at the
    // origin with half-edge `half` on every axis.
    let v = [
        TyVector3F64::from_array(triangle[0]) - center,
        TyVector3F64::from_array(triangle[1]) - center,
        TyVector3F64::from_array(triangle[2]) - center,
    ];

    let edges = [v[1] - v[0], v[2] - v[1], v[0] - v[2]];
    let axes = [
        TyVector3F64::new(1.0, 0.0, 0.0),
        TyVector3F64::new(0.0, 1.0, 0.0),
        TyVector3F64::new(0.0, 0.0, 1.0),
    ];

    // The 9 cross products of a cube axis with a triangle edge.
    for edge in &edges {
        for axis in &axes {
            if separates(axis.cross(edge), &v, half) {
                return false;
            }
        }
    }

    // The 3 cube face normals.
    for axis in axes {
        if separates(axis, &v, half) {
            return false;
        }
    }
    // The triangle normal.
    !separates(edges[0].cross(&edges[1]), &v, half)
}

/// Whether `axis` separates the origin-centered cube of half-edge `half` from
/// triangle `v` (already relative to the cube center). A near-zero axis, as a
/// degenerate edge cross produces, never separates.
fn separates(axis: TyVector3F64, v: &[TyVector3F64; 3], half: f64) -> bool {
    let p0 = axis.dot(&v[0]);
    let p1 = axis.dot(&v[1]);
    let p2 = axis.dot(&v[2]);
    let min = p0.min(p1).min(p2);
    let max = p0.max(p1).max(p2);
    let radius = half * (axis.x.abs() + axis.y.abs() + axis.z.abs());
    min > radius || max < -radius
}

#[cfg(test)]
mod tests {
    use crate::triangle_box_overlap;

    // The unit cube [0, 1]^3.
    const CENTER: [f64; 3] = [0.5, 0.5, 0.5];
    const HALF: f64 = 0.5;

    #[test]
    fn triangle_inside_the_cube_overlaps() {
        let triangle = [[0.2, 0.2, 0.5], [0.8, 0.2, 0.5], [0.5, 0.8, 0.5]];
        assert!(triangle_box_overlap(CENTER, HALF, &triangle));
    }

    #[test]
    fn triangle_far_away_does_not_overlap() {
        let triangle = [[5.0, 5.0, 5.0], [6.0, 5.0, 5.0], [5.0, 6.0, 5.0]];
        assert!(!triangle_box_overlap(CENTER, HALF, &triangle));
    }

    #[test]
    fn triangle_skewered_through_the_cube_overlaps() {
        // A triangle whose vertices all sit outside the cube but whose face
        // passes through it; only an edge-cross axis can separate, so this
        // catches a test that checks only face normals.
        let triangle = [[-2.0, 0.5, 0.5], [2.0, 0.5, -2.0], [2.0, 0.5, 2.0]];
        assert!(triangle_box_overlap(CENTER, HALF, &triangle));
    }

    #[test]
    fn triangle_in_a_parallel_plane_outside_does_not_overlap() {
        // Coplanar offset: the triangle's plane is z = 2, clear of the cube.
        let triangle = [[0.0, 0.0, 2.0], [1.0, 0.0, 2.0], [0.0, 1.0, 2.0]];
        assert!(!triangle_box_overlap(CENTER, HALF, &triangle));
    }

    #[test]
    fn triangle_grazing_a_face_overlaps() {
        // The triangle lies in the plane z = 1, the cube's far face.
        let triangle = [[0.2, 0.2, 1.0], [0.8, 0.2, 1.0], [0.5, 0.8, 1.0]];
        assert!(triangle_box_overlap(CENTER, HALF, &triangle));
    }
}
