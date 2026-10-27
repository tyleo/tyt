use ty_math::TyVector3F64;

/// A bound on the signed distance to the point at `offset` from the ellipsoid
/// with `radii` around the origin. The bound stays tight outside and never
/// reads deeper than the true depth inside.
pub fn ellipsoid_distance(offset: TyVector3F64, radii: TyVector3F64) -> f64 {
    let k0 = (offset / radii).length();
    let k1 = (offset / (radii * radii)).length();

    if k0 >= 1.0 {
        k0 * (k0 - 1.0) / k1
    } else {
        (k0 - 1.0) * radii.min_element()
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{grid3d, shapes_of};
    use branded_id::U32Id;
    use sdfcore::SdfShape3d;
    use std::f64::consts::{PI, TAU};
    use ty_math::TyVector3F64;

    #[test]
    fn an_ellipsoid_bounds_the_distance_and_reads_exact_on_its_axes() {
        let center = TyVector3F64::new(0.1, -0.2, 0.05);
        let radii = TyVector3F64::new(0.7, 0.3, 0.5);
        let shapes = shapes_of(Vec::new(), vec![SdfShape3d::Ellipsoid { center, radii }]);
        let distance = |point| shapes.evaluate(U32Id::from_u32(0), point).distance;

        // The surface sampled densely, whose least distance never reads below
        // the true distance.
        let surface: Vec<TyVector3F64> = (0..=200)
            .flat_map(|i| (0..400).map(move |j| (i, j)))
            .map(|(i, j)| {
                let polar = PI * f64::from(i) / 200.0;
                let azimuth = TAU * f64::from(j) / 400.0;
                center
                    + radii
                        * TyVector3F64::new(
                            polar.sin() * azimuth.cos(),
                            polar.cos(),
                            polar.sin() * azimuth.sin(),
                        )
            })
            .collect();

        for point in grid3d(TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 7) {
            let read = distance(point);
            let to_surface = surface
                .iter()
                .map(|sample| point.distance(*sample))
                .fold(f64::INFINITY, f64::min);
            let inside = ((point - center) / radii).length_squared() <= 1.0;

            assert_eq!(read <= 0.0, inside, "{point}");
            assert!(
                read.abs() <= to_surface + 1e-9,
                "{point}: {read} past {to_surface}"
            );
        }

        for axis in 0..3 {
            let mut point = center;
            point[axis] += 2.0 * radii[axis];
            assert!((distance(point) - radii[axis]).abs() < 1e-12);
        }
    }
}
