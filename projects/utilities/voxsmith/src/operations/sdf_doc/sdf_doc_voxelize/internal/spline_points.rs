use ty_math::TyVector2F64;

/// The segments each span of a spline splits into.
const SEGMENTS: u32 = 16;

/// The points along the centripetal Catmull-Rom curve through `points`, one
/// per segment end. A closed curve's last segment runs back to the first point
/// without repeating it. An open curve extends each end span straight back
/// across its end point.
pub fn spline_points(points: &[TyVector2F64], closed: bool) -> Vec<TyVector2F64> {
    let count = points.len();

    let at = |index: usize| {
        if closed {
            points[(index + count - 1) % count]
        } else if index == 0 {
            points[0] * 2.0 - points[1]
        } else if index > count {
            points[count - 1] * 2.0 - points[count - 2]
        } else {
            points[index - 1]
        }
    };

    let spans = if closed { count } else { count - 1 };
    let mut curve = Vec::with_capacity(spans * SEGMENTS as usize + 1);

    for span in 0..spans {
        let controls = [at(span), at(span + 1), at(span + 2), at(span + 3)];
        curve.push(controls[1]);
        curve.extend(
            (1..SEGMENTS)
                .map(|segment| catmull_rom(controls, f64::from(segment) / f64::from(SEGMENTS))),
        );
    }

    if !closed {
        curve.push(points[count - 1]);
    }

    curve
}

/// The point `fraction` of the way from `controls[1]` to `controls[2]` by the
/// Barry-Goldman pyramid, with knots spaced by the square root of each
/// control's distance from the last.
fn catmull_rom(controls: [TyVector2F64; 4], fraction: f64) -> TyVector2F64 {
    let [p0, p1, p2, p3] = controls;
    let t1 = p0.distance(p1).sqrt();
    let t2 = t1 + p1.distance(p2).sqrt();
    let t3 = t2 + p2.distance(p3).sqrt();
    let t = t1 + (t2 - t1) * fraction;

    let mix = |a: TyVector2F64, b: TyVector2F64, from: f64, to: f64| {
        a * ((to - t) / (to - from)) + b * ((t - from) / (to - from))
    };

    let a1 = mix(p0, p1, 0.0, t1);
    let a2 = mix(p1, p2, t1, t2);
    let a3 = mix(p2, p3, t2, t3);
    let b1 = mix(a1, a2, 0.0, t2);
    let b2 = mix(a2, a3, t1, t3);

    mix(b1, b2, t1, t2)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::spline_points;
    use std::f64::consts::TAU;
    use ty_math::TyVector2F64;

    fn vectors(points: &[(f64, f64)]) -> Vec<TyVector2F64> {
        points
            .iter()
            .map(|(u, v)| TyVector2F64::new(*u, *v))
            .collect()
    }

    #[test]
    fn a_spline_passes_through_each_point() {
        let points = vectors(&[(0.0, 0.0), (1.0, 0.2), (1.4, 1.0), (0.3, 1.5)]);

        let open = spline_points(&points, false);
        assert_eq!(open.len(), 3 * 16 + 1);
        for (index, point) in points.iter().enumerate() {
            assert_eq!(open[index * 16], *point);
        }

        let closed = spline_points(&points, true);
        assert_eq!(closed.len(), 4 * 16);
        for (index, point) in points.iter().enumerate() {
            assert_eq!(closed[index * 16], *point);
        }
    }

    #[test]
    fn an_open_spline_through_two_points_runs_straight() {
        let line = spline_points(&vectors(&[(0.0, 0.0), (2.0, 1.0)]), false);

        for (index, point) in line.iter().enumerate() {
            let expected = TyVector2F64::new(2.0, 1.0) * (index as f64 / 16.0);
            assert!(point.distance(expected) < 1e-12, "{point} at {index}");
        }
    }

    #[test]
    fn a_closed_spline_through_a_circle_s_points_follows_the_circle() {
        let points: Vec<TyVector2F64> = (0..8)
            .map(|index| {
                let (sin, cos) = (f64::from(index) / 8.0 * TAU).sin_cos();
                TyVector2F64::new(cos, sin)
            })
            .collect();

        for point in spline_points(&points, true) {
            assert!((point.length() - 1.0).abs() < 1e-2, "{point}");
        }
    }
}
