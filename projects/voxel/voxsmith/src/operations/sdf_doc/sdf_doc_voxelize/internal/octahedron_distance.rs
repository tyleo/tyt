use ty_math::TyVector3F64;

/// The signed distance to the point at `offset` from the octahedron reaching
/// `radius` along each axis from the origin.
pub fn octahedron_distance(offset: TyVector3F64, radius: f64) -> f64 {
    let p = offset.abs();
    let m = p.x + p.y + p.z - radius;

    let q = if 3.0 * p.x < m {
        p
    } else if 3.0 * p.y < m {
        TyVector3F64::new(p.y, p.z, p.x)
    } else if 3.0 * p.z < m {
        TyVector3F64::new(p.z, p.x, p.y)
    } else {
        return m / 3.0_f64.sqrt();
    };

    let k = (0.5 * (q.z - q.y + radius)).clamp(0.0, radius);
    TyVector3F64::new(q.x, q.y - radius + k, q.z - k).length()
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape3d, triangle_distance};
    use sdfcore::SdfShape3d;
    use ty_math::TyVector3F64;

    #[test]
    fn an_octahedron_measures_exact_distances() {
        let center = TyVector3F64::new(0.1, -0.2, 0.05);
        let radius = 0.6;

        let mut faces = Vec::new();

        for x in [-radius, radius] {
            for y in [-radius, radius] {
                for z in [-radius, radius] {
                    faces.push([
                        center + TyVector3F64::new(x, 0.0, 0.0),
                        center + TyVector3F64::new(0.0, y, 0.0),
                        center + TyVector3F64::new(0.0, 0.0, z),
                    ]);
                }
            }
        }

        let shape = SdfShape3d::Octahedron { center, radius };
        let grid = (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

        assert_shape3d(Vec::new(), vec![shape], grid, 1e-12, |point| {
            let to_surface = faces
                .iter()
                .map(|[a, b, c]| triangle_distance(point, *a, *b, *c))
                .fold(f64::INFINITY, f64::min);
            let offset = (point - center).abs();

            if offset.x + offset.y + offset.z <= radius {
                -to_surface
            } else {
                to_surface
            }
        });
    }
}
