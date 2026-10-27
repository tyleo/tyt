use crate::operations::sdf_doc::{Bounds2d, OutlineContour, OutlineSegment};
use std::f64::consts::TAU;
use ty_math::TyVector2F64;

/// The signed distance to `point` from the outline of `contours`. The sign
/// follows the nonzero rule.
pub fn outline_distance(contours: &[OutlineContour], point: TyVector2F64) -> f64 {
    let mut distance_squared = f64::INFINITY;
    let mut winding = 0;

    // A contour lies inside its box, so it is never nearer than the box and
    // winds around no point outside it. The loop visits the nearest boxes first
    // and stops at a box outside the point that is no nearer than the nearest
    // segment so far.
    let mut nearest: Vec<_> = contours
        .iter()
        .map(|contour| (box_distance_squared(&contour.bounds, point), contour))
        .collect();
    nearest.sort_by(|(a, _), (b, _)| a.total_cmp(b));

    for (box_squared, contour) in nearest {
        if box_squared > 0.0 && box_squared >= distance_squared {
            break;
        }

        let segments = &contour.segments;
        let mut from = segments.last().expect("a contour holds segments").to();

        for segment in segments {
            let to = segment.to();

            let (segment_winding, segment_squared) = match *segment {
                OutlineSegment::Line { .. } => (
                    line_winding(from, to, point),
                    line_distance_squared(from, to, point),
                ),

                OutlineSegment::Quadratic { control, .. } => (
                    quadratic_winding(from, control, to, point),
                    quadratic_distance_squared(from, control, to, point),
                ),
            };
            winding += segment_winding;
            distance_squared = distance_squared.min(segment_squared);
            from = to;
        }
    }

    let distance = distance_squared.sqrt();

    if winding != 0 { -distance } else { distance }
}

/// The squared distance to `point` from `bounds`, which reads zero inside it.
fn box_distance_squared(bounds: &Bounds2d, point: TyVector2F64) -> f64 {
    let outside = (bounds.min - point)
        .max(point - bounds.max)
        .max(TyVector2F64::ZERO);
    outside.length_squared()
}

/// How the line from `from` to `to` winds around `point`, counted where it
/// crosses the ray along +u.
fn line_winding(from: TyVector2F64, to: TyVector2F64, point: TyVector2F64) -> i32 {
    let side = (to - from).perp_dot(point - from);

    if from.y <= point.y && point.y < to.y && side > 0.0 {
        1
    } else if to.y <= point.y && point.y < from.y && side < 0.0 {
        -1
    } else {
        0
    }
}

/// How the quadratic curve winds around `point`, counted where it crosses the
/// ray along +u. The curve splits where it turns in v so that each piece
/// crosses the ray at most once.
fn quadratic_winding(
    from: TyVector2F64,
    control: TyVector2F64,
    to: TyVector2F64,
    point: TyVector2F64,
) -> i32 {
    let at =
        |t: f64| from * ((1.0 - t) * (1.0 - t)) + control * (2.0 * t * (1.0 - t)) + to * (t * t);

    // The curve's v runs `a * t^2 + b * t + c` above the point's.
    let a = from.y - 2.0 * control.y + to.y;
    let b = 2.0 * (control.y - from.y);
    let c = from.y - point.y;
    let residual = |t: f64| ((a * t + b) * t + c).abs();

    let turn = -b / (2.0 * a);
    let turn = if turn > 0.0 && turn < 1.0 { turn } else { 1.0 };

    [(0.0, turn), (turn, 1.0)]
        .into_iter()
        .map(|(start, end)| {
            let (v_start, v_end) = (at(start).y, at(end).y);
            let sign = if v_start <= point.y && point.y < v_end {
                1
            } else if v_end <= point.y && point.y < v_start {
                -1
            } else {
                return 0;
            };

            let roots = if a == 0.0 {
                [-c / b, f64::NAN]
            } else {
                let discriminant = (b * b - 4.0 * a * c).max(0.0);
                let q = -0.5 * (b + discriminant.sqrt().copysign(b));
                [q / a, c / q]
            };
            let t = roots
                .into_iter()
                .filter(|t| !t.is_nan())
                .map(|t| t.clamp(start, end))
                .min_by(|t0, t1| residual(*t0).total_cmp(&residual(*t1)))
                .expect("a piece the ray's line crosses holds a root");

            if at(t).x > point.x { sign } else { 0 }
        })
        .sum()
}

fn line_distance_squared(from: TyVector2F64, to: TyVector2F64, point: TyVector2F64) -> f64 {
    let e = to - from;
    let w = point - from;

    // A contour can repeat a point, which leaves a line with no length.
    let e_squared = e.dot(e);
    let t = if e_squared > 0.0 {
        (w.dot(e) / e_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (w - e * t).length_squared()
}

/// The squared distance to `point` from the quadratic curve. The nearest point
/// lies at an end or at a root of the cubic in `t` where the curve's tangent
/// runs perpendicular to the line to `point`.
fn quadratic_distance_squared(
    from: TyVector2F64,
    control: TyVector2F64,
    to: TyVector2F64,
    point: TyVector2F64,
) -> f64 {
    let a = control - from;
    let b = from - control * 2.0 + to;
    let m = from - point;
    let at = |t: f64| (m + a * (2.0 * t) + b * (t * t)).length_squared();

    cubic_roots(
        b.dot(b),
        3.0 * a.dot(b),
        2.0 * a.dot(a) + m.dot(b),
        m.dot(a),
    )
    .into_iter()
    .filter(|t| (0.0..=1.0).contains(t))
    .fold(at(0.0).min(at(1.0)), |least, t| least.min(at(t)))
}

/// The real roots of `c3 * t^3 + c2 * t^2 + c1 * t + c0`, with NaN in the
/// places of missing ones. Two Newton steps polish each root because the
/// closed forms lose precision.
fn cubic_roots(c3: f64, c2: f64, c1: f64, c0: f64) -> [f64; 3] {
    let mut roots = [f64::NAN; 3];

    if c3.abs() <= 1e-12 * c2.abs().max(c1.abs()).max(c0.abs()) {
        if c2.abs() <= 1e-12 * c1.abs().max(c0.abs()) {
            if c1 != 0.0 {
                roots[0] = -c0 / c1;
            }
        } else {
            let discriminant = c1 * c1 - 4.0 * c2 * c0;
            if discriminant >= 0.0 {
                let q = -0.5 * (c1 + discriminant.sqrt().copysign(c1));
                roots[0] = q / c2;
                if q != 0.0 {
                    roots[1] = c0 / q;
                }
            }
        }
    } else {
        let (a, b, c) = (c2 / c3, c1 / c3, c0 / c3);
        let p = b - a * a / 3.0;
        let q = 2.0 * a * a * a / 27.0 - a * b / 3.0 + c;
        let shift = -a / 3.0;
        let discriminant = q * q / 4.0 + p * p * p / 27.0;

        if discriminant > 0.0 {
            let s = discriminant.sqrt();
            roots[0] = (-q / 2.0 + s).cbrt() + (-q / 2.0 - s).cbrt() + shift;
        } else if p == 0.0 {
            roots[0] = shift;
        } else {
            let r = (-p / 3.0).sqrt();
            let phi = (1.5 * q / p * (-3.0 / p).sqrt()).clamp(-1.0, 1.0).acos();
            for (k, root) in (0..).zip(&mut roots) {
                *root = 2.0 * r * ((phi - TAU * f64::from(k)) / 3.0).cos() + shift;
            }
        }
    }

    roots.map(|mut t| {
        for _ in 0..2 {
            let slope = (3.0 * c3 * t + 2.0 * c2) * t + c1;
            if slope != 0.0 {
                t -= (((c3 * t + c2) * t + c1) * t + c0) / slope;
            }
        }
        t
    })
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        OutlineContour, OutlineSegment, grid2d, outline_distance, polygon_distance,
        polygon_reference,
    };
    use std::slice;
    use ty_math::TyVector2F64;

    fn line(u: f64, v: f64) -> OutlineSegment {
        OutlineSegment::Line {
            to: TyVector2F64::new(u, v),
        }
    }

    fn quadratic(cu: f64, cv: f64, u: f64, v: f64) -> OutlineSegment {
        OutlineSegment::Quadratic {
            control: TyVector2F64::new(cu, cv),
            to: TyVector2F64::new(u, v),
        }
    }

    /// Checks the distance to `contours` over the grid from `min` to `max` with
    /// `steps` points along each axis. Each distance lies within `tolerance` of
    /// `expected`.
    fn assert_outline(
        contours: &[OutlineContour],
        (min, max, steps): (TyVector2F64, TyVector2F64, u32),
        tolerance: f64,
        expected: impl Fn(TyVector2F64) -> f64,
    ) {
        for point in grid2d(min, max, steps) {
            let distance = outline_distance(contours, point);
            let expected = expected(point);

            assert!(
                (distance - expected).abs() <= tolerance,
                "at {point}: read {distance}, expected {expected}",
            );
        }
    }

    fn square(min: f64, max: f64) -> OutlineContour {
        OutlineContour::new(vec![
            line(max, min),
            line(max, max),
            line(min, max),
            line(min, min),
        ])
    }

    #[test]
    fn an_outline_of_lines_measures_as_its_polygon() {
        let points = [(0.2, -0.2), (0.9, 0.0), (0.2, 0.6), (-0.8, 0.2)]
            .map(|(u, v)| TyVector2F64::new(u, v));
        let contour =
            OutlineContour::new(points.iter().map(|point| line(point.x, point.y)).collect());
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

        assert_outline(&[contour], grid, 1e-12, |point| {
            polygon_distance(&points, point)
        });
    }

    #[test]
    fn an_outline_measures_exact_distances_to_curves_bowing_either_way() {
        let contour = vec![
            line(0.8, -0.8),
            line(0.8, 0.8),
            quadratic(0.0, 0.0, -0.8, 0.8),
            quadratic(-1.4, 0.0, -0.8, -0.8),
        ];
        let start = TyVector2F64::splat(-0.8);
        let mut flattened = vec![start];
        let mut from = start;
        for segment in &contour {
            match *segment {
                OutlineSegment::Line { to } => flattened.push(to),

                OutlineSegment::Quadratic { control, to } => {
                    flattened.extend((1..=4000).map(|step| {
                        let t = f64::from(step) / 4000.0;
                        from * ((1.0 - t) * (1.0 - t))
                            + control * (2.0 * t * (1.0 - t))
                            + to * (t * t)
                    }));
                }
            }
            from = segment.to();
        }
        flattened.pop();

        let grid = (TyVector2F64::splat(-1.2), TyVector2F64::splat(1.2), 25);

        assert_outline(&[OutlineContour::new(contour)], grid, 1e-6, |point| {
            polygon_reference(&flattened, point)
        });
    }

    #[test]
    fn a_straight_curve_measures_as_its_line() {
        let curved = OutlineContour::new(vec![
            line(1.0, 0.0),
            quadratic(1.0, 0.5, 1.0, 1.0),
            line(0.0, 1.0),
            line(0.0, 0.0),
        ]);
        let point = TyVector2F64::new(1.3, 0.4);

        assert!((outline_distance(&[curved], point) - 0.3).abs() <= 1e-12);
    }

    #[test]
    fn the_sign_follows_the_nonzero_rule() {
        let hole = OutlineContour::new(vec![
            line(-0.25, 0.25),
            line(0.25, 0.25),
            line(0.25, -0.25),
            line(-0.25, -0.25),
        ]);
        let ring = [square(-1.0, 1.0), hole];
        let overlap = [square(-1.0, 0.5), square(-0.5, 1.0)];

        assert!(outline_distance(&ring, TyVector2F64::ZERO) > 0.0);
        assert!(outline_distance(&ring, TyVector2F64::splat(0.5)) < 0.0);
        assert!(outline_distance(&overlap, TyVector2F64::ZERO) < 0.0);
    }

    #[test]
    fn a_row_of_contours_measures_as_its_nearest() {
        let row: Vec<_> = (0..6)
            .map(|index| square(f64::from(index), f64::from(index) + 0.5))
            .collect();
        let grid = (TyVector2F64::splat(-1.0), TyVector2F64::new(7.0, 1.5), 41);

        assert_outline(&row, grid, 0.0, |point| {
            row.iter()
                .map(|contour| outline_distance(slice::from_ref(contour), point))
                .fold(f64::INFINITY, f64::min)
        });
    }
}
