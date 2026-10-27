use ty_math::TyVector3F64;

/// The signed distance to the point at `offset` from the pyramid on the unit
/// square base around the origin, with its apex `height` up +y.
pub fn pyramid_distance(offset: TyVector3F64, height: f64) -> f64 {
    let x = offset.x.abs();
    let z = offset.z.abs();

    // At or below the base plane the base square lies nearest.
    if offset.y <= 0.0 {
        return TyVector3F64::new((x - 0.5).max(0.0), offset.y, (z - 0.5).max(0.0)).length();
    }

    let m2 = height * height + 0.25;
    let (px, pz) = if z > x { (z, x) } else { (x, z) };
    let px = px - 0.5;
    let pz = pz - 0.5;
    let py = offset.y;

    let qx = pz;
    let qy = height * py - 0.5 * px;
    let qz = height * px + 0.5 * py;

    let s = (-qx).max(0.0);
    let t = ((qy - 0.5 * pz) / (m2 + 0.25)).clamp(0.0, 1.0);
    let a = m2 * (qx + s) * (qx + s) + qy * qy;
    let b = m2 * (qx + 0.5 * t) * (qx + 0.5 * t) + (qy - m2 * t) * (qy - m2 * t);
    let d2 = if qy.min(-qx * m2 - qy * 0.5) > 0.0 {
        0.0
    } else {
        a.min(b)
    };

    let to_faces = ((d2 + qz * qz) / m2).sqrt();

    // Inside, the base can lie nearer than the slanted faces.
    if qz < 0.0 {
        -to_faces.min(py)
    } else {
        to_faces
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape3d, triangle_distance};
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    #[test]
    fn a_pyramid_measures_exact_distances_below_and_inside_its_base() {
        let base_center = TyVector3F64::new(0.1, -0.2, 0.05);
        let grid = (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

        for (width, height) in [(1.0, 0.8), (0.5, 1.1)] {
            let half = width / 2.0;
            let corner = |x: f64, z: f64| base_center + TyVector3F64::new(x, 0.0, z);
            let apex = base_center + TyVector3F64::new(0.0, height, 0.0);
            let corners = [
                corner(-half, -half),
                corner(half, -half),
                corner(half, half),
                corner(-half, half),
            ];

            let mut faces: Vec<[TyVector3F64; 3]> = (0..4)
                .map(|index| [corners[index], corners[(index + 1) % 4], apex])
                .collect();
            faces.push([corners[0], corners[1], corners[2]]);
            faces.push([corners[0], corners[2], corners[3]]);

            let shape = SdfShape3d::Pyramid {
                base_center,
                width,
                height,
            };

            assert_shape3d(Vec::new(), vec![shape], grid, 1e-12, |point| {
                let to_surface = faces
                    .iter()
                    .map(|[a, b, c]| triangle_distance(point, *a, *b, *c))
                    .fold(f64::INFINITY, f64::min);
                let offset = point - base_center;
                let reach = half * (1.0 - offset.y / height);
                let inside = offset.y >= 0.0 && offset.x.abs() <= reach && offset.z.abs() <= reach;

                if inside { -to_surface } else { to_surface }
            });
        }
    }
}
