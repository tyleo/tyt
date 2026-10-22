use crate::operations::sdf_doc::{ArgumentCheck, spline_points};
use sdfcore::SdfShape2d;
use std::result::Result as StdResult;
use ty_math::TyVector2F64;

/// Checks the arguments of the 2D shape `shape`.
pub fn check_shape2d(shape: &SdfShape2d) -> StdResult<(), String> {
    match shape {
        SdfShape2d::Arc {
            radius,
            from_degrees,
            to_degrees,
            width,
            ..
        } => {
            let check = ArgumentCheck::new("arc");
            check.above_zero("radius", *radius)?;
            check.above_zero("width", *width)?;
            span(check, *from_degrees, *to_degrees)
        }

        SdfShape2d::Arch { min, max } => {
            let check = ArgumentCheck::new("arch");
            check.corners2d(*min, *max)?;

            let half_width = (max.x - min.x) / 2.0;
            check.expect(
                max.y - min.y >= half_width,
                "max",
                &format!("at least half the width, {half_width}, above min"),
                format!("[{}, {}]", max.x, max.y),
            )
        }

        SdfShape2d::Circle { radius, .. } => {
            ArgumentCheck::new("circle").above_zero("radius", *radius)
        }

        SdfShape2d::Ellipse { radii, .. } => {
            let check = ArgumentCheck::new("ellipse");
            check.expect(
                radii.cmpgt(TyVector2F64::ZERO).all(),
                "radii",
                "above zero on each axis",
                format!("[{}, {}]", radii.x, radii.y),
            )
        }

        SdfShape2d::Intersect { shape_ids } => {
            ArgumentCheck::new("intersect").count("shapes", shape_ids.len(), 1)
        }

        SdfShape2d::Mirror { .. }
        | SdfShape2d::Offset { .. }
        | SdfShape2d::Rotate { .. }
        | SdfShape2d::Subtract { .. }
        | SdfShape2d::Translate { .. } => Ok(()),

        SdfShape2d::Ngon { sides, radius, .. } => {
            let check = ArgumentCheck::new("ngon");
            check.whole_at_least("sides", *sides, 3.0)?;
            check.above_zero("radius", *radius)
        }

        SdfShape2d::Polygon { points } => {
            let check = ArgumentCheck::new("polygon");
            check.count("points", points.len(), 3)?;
            check.expect(
                !crosses_itself(points),
                "points",
                "an outline that never crosses itself",
                "an outline that crosses itself",
            )
        }

        SdfShape2d::Polyline { points, width } => {
            let check = ArgumentCheck::new("polyline");
            check.count("points", points.len(), 2)?;
            check.above_zero("width", *width)?;
            distinct_neighbors(&check, points, false)
        }

        SdfShape2d::Rect {
            min,
            max,
            chamfer,
            round,
        } => {
            let check = ArgumentCheck::new("rect");
            check.opposite_corners2d(*min, *max)?;

            let most = (*max - *min).abs().min_element() / 2.0;

            match (chamfer, round) {
                (Some(_), Some(round)) => {
                    Err(check.fail("round", "left out beside chamfer", round))
                }

                (Some(corner), None) => corner_cut(check, "chamfer", *corner, most),

                (None, Some(corner)) => corner_cut(check, "round", *corner, most),

                (None, None) => Ok(()),
            }
        }

        SdfShape2d::Repeat { count, .. } => {
            let check = ArgumentCheck::new("repeat");
            check.whole_above_zero("count", count.x)?;
            check.whole_above_zero("count", count.y)
        }

        SdfShape2d::RepeatPolar { count, .. } => {
            ArgumentCheck::new("repeatPolar").whole_above_zero("count", *count)
        }

        SdfShape2d::Scale { factor, .. } => {
            let check = ArgumentCheck::new("scale");
            check.expect(
                factor.cmpne(TyVector2F64::ZERO).all(),
                "factor",
                "nonzero on each axis",
                format!("[{}, {}]", factor.x, factor.y),
            )
        }

        SdfShape2d::Sector {
            radius,
            from_degrees,
            to_degrees,
            ..
        } => {
            let check = ArgumentCheck::new("sector");
            check.above_zero("radius", *radius)?;
            span(check, *from_degrees, *to_degrees)
        }

        SdfShape2d::Shell { thickness, .. } => {
            ArgumentCheck::new("shell").above_zero("thickness", *thickness)
        }

        SdfShape2d::SmoothIntersect { radius, shape_ids } => {
            let check = ArgumentCheck::new("smoothIntersect");
            check.above_zero("radius", *radius)?;
            check.count("shapes", shape_ids.len(), 1)
        }

        SdfShape2d::SmoothSubtract { radius, .. } => {
            ArgumentCheck::new("smoothSubtract").above_zero("radius", *radius)
        }

        SdfShape2d::SmoothUnion { radius, shape_ids } => {
            let check = ArgumentCheck::new("smoothUnion");
            check.above_zero("radius", *radius)?;
            check.count("shapes", shape_ids.len(), 1)
        }

        SdfShape2d::Spline {
            points,
            width: None,
        } => {
            let check = ArgumentCheck::new("spline");
            check.count("points", points.len(), 3)?;
            distinct_neighbors(&check, points, true)?;
            check.expect(
                !crosses_itself(&spline_points(points, true, TyVector2F64::distance)),
                "points",
                "a curve that never crosses itself",
                "a curve that crosses itself",
            )
        }

        SdfShape2d::Spline {
            points,
            width: Some(width),
        } => {
            let check = ArgumentCheck::new("spline");
            check.count("points", points.len(), 2)?;
            check.above_zero("width", *width)?;
            distinct_neighbors(&check, points, false)
        }

        SdfShape2d::Star {
            points,
            outer_radius,
            inner_radius,
            ..
        } => {
            let check = ArgumentCheck::new("star");
            check.whole_at_least("points", *points, 2.0)?;
            check.above_zero("outerRadius", *outer_radius)?;
            check.above_zero("innerRadius", *inner_radius)?;
            check.expect(
                inner_radius < outer_radius,
                "innerRadius",
                &format!("below outerRadius {outer_radius}"),
                inner_radius,
            )
        }

        SdfShape2d::Union { shape_ids } => {
            ArgumentCheck::new("union").count("shapes", shape_ids.len(), 1)
        }

        SdfShape2d::Vesica { a, b, width } => {
            let check = ArgumentCheck::new("vesica");
            check.above_zero("width", *width)?;
            check.expect(
                a != b,
                "b",
                "another point than a",
                format!("[{}, {}]", b.x, b.y),
            )
        }
    }
}

/// Errors unless the span from `from` to `to` degrees runs forward by at most
/// a full turn.
fn span(check: ArgumentCheck, from: f64, to: f64) -> StdResult<(), String> {
    check.range(from, to)?;
    check.expect(
        to - from <= 360.0,
        "toDegrees",
        &format!("at most 360 past fromDegrees {from}"),
        to,
    )
}

/// Errors when two neighboring `points` match. The last point neighbors the
/// first when `closed`.
fn distinct_neighbors(
    check: &ArgumentCheck,
    points: &[TyVector2F64],
    closed: bool,
) -> StdResult<(), String> {
    let wrap = closed.then(|| [points[points.len() - 1], points[0]]);

    match points
        .windows(2)
        .map(|pair| [pair[0], pair[1]])
        .chain(wrap)
        .find(|pair| pair[0] == pair[1])
    {
        Some(pair) => Err(check.fail(
            "points",
            "distinct from their neighbors",
            format!("[{}, {}] twice in a row", pair[0].x, pair[0].y),
        )),

        None => Ok(()),
    }
}

/// Errors unless a rect's corner cut `corner` reads zero or more and at most
/// `most`.
fn corner_cut(
    check: ArgumentCheck,
    argument: &str,
    corner: f64,
    most: f64,
) -> StdResult<(), String> {
    check.at_least_zero(argument, corner)?;
    check.at_most(
        argument,
        corner,
        most,
        &format!("at most half the shorter side, {most}"),
    )
}

/// Whether the closed outline through `points` crosses or touches itself.
/// Repeated neighboring points count as one.
fn crosses_itself(points: &[TyVector2F64]) -> bool {
    let mut outline: Vec<TyVector2F64> = Vec::with_capacity(points.len());

    for &point in points {
        if outline.last() != Some(&point) {
            outline.push(point);
        }
    }

    while outline.len() > 1 && outline.first() == outline.last() {
        outline.pop();
    }

    let count = outline.len();
    let edge = |index: usize| (outline[index], outline[(index + 1) % count]);

    (0..count).any(|first| {
        (first + 1..count).any(|second| {
            let (a, b) = edge(first);
            let (c, d) = edge(second);

            if second == first + 1 || (first == 0 && second == count - 1) {
                // Neighboring edges share one point and cross only by folding
                // back along each other.
                let (shared, before, after) = if second == first + 1 {
                    (b, a, d)
                } else {
                    (a, b, c)
                };

                cross(before, shared, after) == 0.0 && (after - shared).dot(before - shared) > 0.0
            } else {
                segments_meet(a, b, c, d)
            }
        })
    })
}

/// Whether the segments from `a` to `b` and from `c` to `d` share a point.
fn segments_meet(a: TyVector2F64, b: TyVector2F64, c: TyVector2F64, d: TyVector2F64) -> bool {
    let sides = [
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    ];

    if sides[0] * sides[1] < 0.0 && sides[2] * sides[3] < 0.0 {
        return true;
    }

    let on = |p: TyVector2F64, q: TyVector2F64, r: TyVector2F64, side: f64| {
        side == 0.0
            && r.x >= p.x.min(q.x)
            && r.x <= p.x.max(q.x)
            && r.y >= p.y.min(q.y)
            && r.y <= p.y.max(q.y)
    };

    on(a, b, c, sides[0]) || on(a, b, d, sides[1]) || on(c, d, a, sides[2]) || on(c, d, b, sides[3])
}

/// The cross product of `b - a` and `c - a`, positive when `c` lies left of
/// the line from `a` to `b`.
fn cross(a: TyVector2F64, b: TyVector2F64, c: TyVector2F64) -> f64 {
    (b - a).perp_dot(c - a)
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_shape2d;
    use sdfcore::SdfShape2d;
    use ty_math::TyVector2F64;

    fn polygon(points: &[(f64, f64)]) -> SdfShape2d {
        SdfShape2d::Polygon {
            points: points
                .iter()
                .map(|(u, v)| TyVector2F64::new(*u, *v))
                .collect(),
        }
    }

    #[test]
    fn a_polygon_may_not_cross_itself() {
        assert!(check_shape2d(&polygon(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)])).is_ok());
        assert!(
            check_shape2d(&polygon(&[
                (0.0, 0.0),
                (1.0, 0.0),
                (1.0, 0.0),
                (0.5, 1.0),
                (0.0, 0.0)
            ]))
            .is_ok()
        );

        let bow_tie = check_shape2d(&polygon(&[(0.0, 0.0), (1.0, 1.0), (1.0, 0.0), (0.0, 1.0)]));
        assert_eq!(
            bow_tie,
            Err("polygon points must be an outline that never crosses itself, not an outline that crosses itself".to_string())
        );

        // A spike folding back along its edge touches itself.
        assert!(
            check_shape2d(&polygon(&[(0.0, 0.0), (2.0, 0.0), (1.0, 0.0), (1.0, 1.0)])).is_err()
        );
    }

    #[test]
    fn a_spline_lists_distinct_neighbors_and_its_outline_never_crosses_itself() {
        let spline = |points: &[(f64, f64)], width: Option<f64>| SdfShape2d::Spline {
            points: points
                .iter()
                .map(|(u, v)| TyVector2F64::new(*u, *v))
                .collect(),
            width,
        };
        let square = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

        assert!(check_shape2d(&spline(&square, None)).is_ok());
        assert!(check_shape2d(&spline(&square[..2], Some(0.1))).is_ok());
        assert_eq!(
            check_shape2d(&spline(&square[..2], None)),
            Err("spline points must be a list of at least 3, not a list of 2".to_string())
        );
        assert_eq!(
            check_shape2d(&spline(
                &[(0.0, 0.0), (1.0, 0.0), (0.5, 1.0), (0.0, 0.0)],
                None
            )),
            Err(
                "spline points must be distinct from their neighbors, not [0, 0] twice in a row"
                    .to_string()
            )
        );
        assert_eq!(
            check_shape2d(&spline(&[(0.0, 0.0), (1.0, 1.0), (1.0, 0.0), (0.0, 1.0)], None)),
            Err(
                "spline points must be a curve that never crosses itself, not a curve that crosses itself"
                    .to_string()
            )
        );
        assert_eq!(
            check_shape2d(&spline(&[(0.0, 0.0), (0.0, 0.0)], Some(0.1))),
            Err(
                "spline points must be distinct from their neighbors, not [0, 0] twice in a row"
                    .to_string()
            )
        );
    }

    #[test]
    fn a_rect_takes_one_corner_cut_within_half_its_shorter_side() {
        let rect = |chamfer: Option<f64>, round: Option<f64>| SdfShape2d::Rect {
            min: TyVector2F64::ZERO,
            max: TyVector2F64::new(1.0, 0.5),
            chamfer,
            round,
        };

        assert!(check_shape2d(&rect(Some(0.25), None)).is_ok());
        assert!(check_shape2d(&rect(Some(0.0), None)).is_ok());
        assert!(check_shape2d(&rect(None, Some(0.0))).is_ok());
        assert_eq!(
            check_shape2d(&rect(None, Some(-0.1))),
            Err("rect round must be zero or more, not -0.1".to_string())
        );
        let flipped = SdfShape2d::Rect {
            min: TyVector2F64::new(1.0, 0.5),
            max: TyVector2F64::ZERO,
            chamfer: Some(0.25),
            round: None,
        };
        assert!(check_shape2d(&flipped).is_ok());
        assert_eq!(
            check_shape2d(&rect(None, Some(0.3))),
            Err("rect round must be at most half the shorter side, 0.25, not 0.3".to_string())
        );
        assert_eq!(
            check_shape2d(&rect(Some(0.1), Some(0.1))),
            Err("rect round must be left out beside chamfer, not 0.1".to_string())
        );
    }
}
