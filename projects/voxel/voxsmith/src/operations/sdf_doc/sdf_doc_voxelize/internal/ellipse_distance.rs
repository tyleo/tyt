use ty_math::TyVector2F64;

/// The Newton steps that refine the nearest point on an ellipse.
const NEWTON_STEPS: usize = 10;

/// The signed distance to the point at `offset` from the ellipse with `radii`
/// around the origin.
pub fn ellipse_distance(offset: TyVector2F64, radii: TyVector2F64) -> f64 {
    if radii.x == radii.y {
        return offset.length() - radii.x;
    }

    let p = offset.abs();

    // The cosine and sine of the nearest point's parameter, starting near the
    // end of the axis the point lies nearer.
    let q = radii * (p - radii);
    let mut cs = if q.x < q.y {
        TyVector2F64::new(0.01, 1.0)
    } else {
        TyVector2F64::new(1.0, 0.01)
    };
    cs /= cs.length();

    for _ in 0..NEWTON_STEPS {
        let u = radii * cs;
        let v = radii * TyVector2F64::new(-cs.y, cs.x);
        let a = (p - u).dot(v);
        let c = (p - u).dot(u) + v.dot(v);
        let b = (c * c - a * a).sqrt();
        cs = TyVector2F64::new(cs.x * b - cs.y * a, cs.y * b + cs.x * a) / c;
    }

    let distance = (p - radii * cs).length();
    let (a, b) = (radii.x, radii.y);

    if b * b * p.x * p.x + a * a * p.y * p.y <= a * a * b * b {
        -distance
    } else {
        distance
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{assert_shape2d, ellipse_distance, grid2d};
    use sdfcore::SdfShape2d;
    use std::f64::consts::TAU;
    use ty_math::TyVector2F64;

    #[test]
    fn an_ellipse_measures_to_its_nearest_point() {
        let center = TyVector2F64::new(0.1, -0.2);
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        for radii in [
            TyVector2F64::new(0.7, 0.3),
            TyVector2F64::new(0.2, 0.6),
            TyVector2F64::new(0.5, 0.5),
        ] {
            // The outline sampled densely enough to land within 2e-5 of every
            // point on it.
            let outline: Vec<TyVector2F64> = (0..100_000)
                .map(|index| {
                    let angle = TAU * f64::from(index) / 100_000.0;
                    center + radii * TyVector2F64::new(angle.cos(), angle.sin())
                })
                .collect();

            let shape = SdfShape2d::Ellipse { center, radii };

            assert_shape2d(vec![shape], grid, 2e-5, |point| {
                let to_outline = outline
                    .iter()
                    .map(|sample| point.distance(*sample))
                    .fold(f64::INFINITY, f64::min);

                if ((point - center) / radii).length_squared() <= 1.0 {
                    -to_outline
                } else {
                    to_outline
                }
            });
        }
    }

    #[test]
    fn narrow_ellipses_read_finite_with_the_division_free_sign() {
        for radii in [
            TyVector2F64::new(1.0, 0.05),
            TyVector2F64::new(0.05, 1.0),
            TyVector2F64::new(1.0, 0.999),
            TyVector2F64::new(0.3, 0.9),
        ] {
            let (a, b) = (radii.x, radii.y);

            for point in grid2d(TyVector2F64::splat(-2.0), TyVector2F64::splat(2.0), 201) {
                let distance = ellipse_distance(point, radii);
                let inside = b * b * point.x * point.x + a * a * point.y * point.y <= a * a * b * b;

                assert!(distance.is_finite(), "{point} on {radii}");
                assert!(
                    if inside {
                        distance <= 0.0
                    } else {
                        distance >= 0.0
                    },
                    "{point} on {radii}"
                );
            }
        }
    }
}
