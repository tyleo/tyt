use crate::operations::sdf_doc::{
    ArcSpan, Bounds2d, OutlineContour, PointMap2d, ShapeSource, arc_distance, arch_distance,
    chamfer_rect_distance, ellipse_distance, exact_sin_cos, fold_to_direction, outline_distance,
    polygon_distance, polyline_distance, rect_distance, sector_distance, smooth_intersect_distance,
    smooth_union_distance, spline_points, vesica_distance, whole_count,
};
use branded_id::{IdVec, U32Id};
use sdfcore::{BSdfShape2d, SdfAxes2d, SdfCaps, SdfShape2d};
use ty_math::TyVector2F64;

/// A 2D shape prepared for evaluation, with every value that holds for all
/// points computed once.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape2dField {
    /// A band along a circle within a span.
    Arc {
        center: TyVector2F64,

        radius: f64,

        half_width: f64,

        from_degrees: f64,

        to_degrees: f64,

        span: ArcSpan,

        caps: SdfCaps,
    },

    /// A rectangle topped with a half circle.
    Arch {
        min: TyVector2F64,

        max: TyVector2F64,
    },

    /// A rectangle with its corners cut at 45 degrees.
    ChamferRect {
        min: TyVector2F64,

        max: TyVector2F64,

        chamfer: f64,
    },

    /// A circle.
    Circle { center: TyVector2F64, radius: f64 },

    /// The union of a shape's copies, one per map.
    Copies {
        shape_id: U32Id<BSdfShape2d>,

        maps: Vec<PointMap2d>,
    },

    /// An ellipse.
    Ellipse {
        center: TyVector2F64,

        radii: TyVector2F64,
    },

    /// The area every shape covers.
    Intersect { shape_ids: Vec<U32Id<BSdfShape2d>> },

    /// A regular polygon, with the unit direction from its center to each
    /// side's midpoint.
    Ngon {
        center: TyVector2F64,

        radius: f64,

        half_side: f64,

        directions: Vec<TyVector2F64>,
    },

    /// A shape grown or shrunk.
    Offset {
        shape_id: U32Id<BSdfShape2d>,

        distance: f64,
    },

    /// Closed contours of lines and quadratic curves, filled by the nonzero
    /// rule.
    Outline { contours: Vec<OutlineContour> },

    /// A closed outline.
    Polygon { points: Vec<TyVector2F64> },

    /// An open line with a width.
    Polyline {
        points: Vec<TyVector2F64>,

        half_width: f64,
    },

    /// A rectangle with rounded corners of radius `round`, which reads 0 for
    /// square corners.
    Rect {
        min: TyVector2F64,

        max: TyVector2F64,

        round: f64,
    },

    /// A slice of a disk within a span.
    Sector {
        center: TyVector2F64,

        radius: f64,

        from_degrees: f64,

        to_degrees: f64,

        span: ArcSpan,
    },

    /// A shape's outer wall.
    Shell {
        shape_id: U32Id<BSdfShape2d>,

        thickness: f64,
    },

    /// An intersection with rounded corners.
    SmoothIntersect {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A subtraction with rounded corners.
    SmoothSubtract {
        radius: f64,

        base_id: U32Id<BSdfShape2d>,

        cutter_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A union with filleted corners.
    SmoothUnion {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A star, with the unit direction from its center to each tip. The frame
    /// that turns a tip onto +v and folds u to `abs(u)` holds the neighboring
    /// inner vertex at `inner_vertex`.
    Star {
        center: TyVector2F64,

        outer_radius: f64,

        inner_radius: f64,

        inner_vertex: TyVector2F64,

        directions: Vec<TyVector2F64>,
    },

    /// A base shape with the cutters removed.
    Subtract {
        base_id: U32Id<BSdfShape2d>,

        cutter_ids: Vec<U32Id<BSdfShape2d>>,
    },

    /// A shape moved, turned, or scaled, whose distance scales by
    /// `distance_factor`.
    Transform {
        shape_id: U32Id<BSdfShape2d>,

        map: PointMap2d,

        distance_factor: f64,
    },

    /// The area any shape covers.
    Union { shape_ids: Vec<U32Id<BSdfShape2d>> },

    /// A pointed lens.
    Vesica {
        a: TyVector2F64,

        b: TyVector2F64,

        half_width: f64,
    },
}

impl Shape2dField {
    /// The field of `shape`, which can't be a `text`.
    pub fn new(shape: &SdfShape2d) -> Self {
        match shape {
            SdfShape2d::Arc {
                center,
                radius,
                from_degrees,
                to_degrees,
                width,
                caps,
            } => Shape2dField::Arc {
                center: *center,
                radius: *radius,
                half_width: width / 2.0,
                from_degrees: *from_degrees,
                to_degrees: *to_degrees,
                span: ArcSpan::new(*from_degrees, *to_degrees),
                caps: caps.unwrap_or(SdfCaps::Round),
            },

            SdfShape2d::Arch { min, max } => Shape2dField::Arch {
                min: *min,
                max: *max,
            },

            SdfShape2d::Circle { center, radius } => Shape2dField::Circle {
                center: *center,
                radius: *radius,
            },

            SdfShape2d::Ellipse { center, radii } => Shape2dField::Ellipse {
                center: *center,
                radii: *radii,
            },

            SdfShape2d::Intersect { shape_ids } => Shape2dField::Intersect {
                shape_ids: shape_ids.clone(),
            },

            SdfShape2d::Mirror {
                shape_id,
                axes,
                center,
            } => Shape2dField::Copies {
                shape_id: *shape_id,
                maps: mirror_maps(*axes, center.unwrap_or(TyVector2F64::ZERO)),
            },

            SdfShape2d::Ngon {
                center,
                sides,
                radius,
            } => {
                let sides = whole_count(*sides);
                let (sin, cos) = exact_sin_cos(180.0 / f64::from(sides));

                Shape2dField::Ngon {
                    center: *center,
                    radius: *radius,
                    half_side: radius * sin / cos,
                    directions: directions(270.0, sides),
                }
            }

            SdfShape2d::Offset { shape_id, distance } => Shape2dField::Offset {
                shape_id: *shape_id,
                distance: *distance,
            },

            SdfShape2d::Polygon { points } => Shape2dField::Polygon {
                points: points.clone(),
            },

            SdfShape2d::Polyline { points, width } => Shape2dField::Polyline {
                points: points.clone(),
                half_width: width / 2.0,
            },

            SdfShape2d::Rect {
                min,
                max,
                chamfer,
                round,
            } => match (chamfer, round) {
                (Some(_), Some(_)) => panic!("a rect takes round or chamfer but not both"),

                // A rect takes its corners in either order on each axis.
                (Some(chamfer), None) => Shape2dField::ChamferRect {
                    min: min.min(*max),
                    max: min.max(*max),
                    chamfer: *chamfer,
                },

                (None, round) => Shape2dField::Rect {
                    min: min.min(*max),
                    max: min.max(*max),
                    round: round.unwrap_or(0.0),
                },
            },

            SdfShape2d::Repeat {
                shape_id,
                step,
                count,
            } => {
                let counts = [whole_count(count.x), whole_count(count.y)];

                Shape2dField::Copies {
                    shape_id: *shape_id,
                    maps: (0..counts[0])
                        .flat_map(|i| (0..counts[1]).map(move |j| (i, j)))
                        .map(|(i, j)| PointMap2d::Translation {
                            offset: TyVector2F64::new(f64::from(i), f64::from(j)) * *step,
                        })
                        .collect(),
                }
            }

            SdfShape2d::RepeatPolar {
                shape_id,
                count,
                center,
            } => {
                let count = whole_count(*count);
                let pivot = center.unwrap_or(TyVector2F64::ZERO);

                Shape2dField::Copies {
                    shape_id: *shape_id,
                    maps: (0..count)
                        .map(|index| {
                            let (sin, cos) =
                                exact_sin_cos(f64::from(index) * 360.0 / f64::from(count));
                            PointMap2d::Rotation { sin, cos, pivot }
                        })
                        .collect(),
                }
            }

            SdfShape2d::Rotate {
                shape_id,
                degrees,
                pivot,
            } => {
                let (sin, cos) = exact_sin_cos(*degrees);

                Shape2dField::Transform {
                    shape_id: *shape_id,
                    map: PointMap2d::Rotation {
                        sin,
                        cos,
                        pivot: pivot.unwrap_or(TyVector2F64::ZERO),
                    },
                    distance_factor: 1.0,
                }
            }

            SdfShape2d::Scale {
                shape_id,
                factor,
                pivot,
            } => Shape2dField::Transform {
                shape_id: *shape_id,
                map: PointMap2d::Scaling {
                    factor: *factor,
                    pivot: pivot.unwrap_or(TyVector2F64::ZERO),
                },
                distance_factor: factor.abs().min_element(),
            },

            SdfShape2d::Sector {
                center,
                radius,
                from_degrees,
                to_degrees,
            } => Shape2dField::Sector {
                center: *center,
                radius: *radius,
                from_degrees: *from_degrees,
                to_degrees: *to_degrees,
                span: ArcSpan::new(*from_degrees, *to_degrees),
            },

            SdfShape2d::Shell {
                shape_id,
                thickness,
            } => Shape2dField::Shell {
                shape_id: *shape_id,
                thickness: *thickness,
            },

            SdfShape2d::SmoothIntersect { radius, shape_ids } => Shape2dField::SmoothIntersect {
                radius: *radius,
                shape_ids: shape_ids.clone(),
            },

            SdfShape2d::SmoothSubtract {
                radius,
                base_id,
                cutter_ids,
            } => Shape2dField::SmoothSubtract {
                radius: *radius,
                base_id: *base_id,
                cutter_ids: cutter_ids.clone(),
            },

            SdfShape2d::SmoothUnion { radius, shape_ids } => Shape2dField::SmoothUnion {
                radius: *radius,
                shape_ids: shape_ids.clone(),
            },

            SdfShape2d::Spline {
                points,
                width: None,
            } => Shape2dField::Polygon {
                points: spline_points(points, true, TyVector2F64::distance),
            },

            SdfShape2d::Spline {
                points,
                width: Some(width),
            } => Shape2dField::Polyline {
                points: spline_points(points, false, TyVector2F64::distance),
                half_width: width / 2.0,
            },

            SdfShape2d::Star {
                center,
                points,
                outer_radius,
                inner_radius,
            } => {
                let points = whole_count(*points);
                let (sin, cos) = exact_sin_cos(180.0 / f64::from(points));

                Shape2dField::Star {
                    center: *center,
                    outer_radius: *outer_radius,
                    inner_radius: *inner_radius,
                    inner_vertex: TyVector2F64::new(sin, cos) * *inner_radius,
                    directions: directions(90.0, points),
                }
            }

            SdfShape2d::Subtract {
                base_id,
                cutter_ids,
            } => Shape2dField::Subtract {
                base_id: *base_id,
                cutter_ids: cutter_ids.clone(),
            },

            SdfShape2d::Translate { shape_id, offset } => Shape2dField::Transform {
                shape_id: *shape_id,
                map: PointMap2d::Translation { offset: *offset },
                distance_factor: 1.0,
            },

            SdfShape2d::Union { shape_ids } => Shape2dField::Union {
                shape_ids: shape_ids.clone(),
            },

            SdfShape2d::Text { .. } => {
                unreachable!("a text's field takes the contours its font lays out")
            }

            SdfShape2d::Vesica { a, b, width } => Shape2dField::Vesica {
                a: *a,
                b: *b,
                half_width: width / 2.0,
            },
        }
    }

    /// The signed distance of the shape at `point`. `shapes` holds the shapes
    /// the field references.
    pub fn distance(&self, shapes: &impl ShapeSource, point: TyVector2F64) -> f64 {
        match self {
            Shape2dField::Arc {
                center,
                radius,
                half_width,
                span,
                caps,
                ..
            } => arc_distance(point - *center, *radius, *half_width, span, *caps),

            Shape2dField::Arch { min, max } => arch_distance(point, *min, *max),

            Shape2dField::ChamferRect { min, max, chamfer } => {
                chamfer_rect_distance(point - (*min + *max) / 2.0, (*max - *min) / 2.0, *chamfer)
            }

            Shape2dField::Circle { center, radius } => (point - *center).length() - radius,

            Shape2dField::Copies { shape_id, maps } => maps
                .iter()
                .map(|map| shapes.distance2d(*shape_id, map.child_point(point)))
                .fold(f64::INFINITY, f64::min),

            Shape2dField::Ellipse { center, radii } => ellipse_distance(point - *center, *radii),

            Shape2dField::Intersect { shape_ids } => shape_ids
                .iter()
                .map(|shape_id| shapes.distance2d(*shape_id, point))
                .reduce(f64::max)
                .expect("an intersect holds a shape"),

            Shape2dField::Ngon {
                center,
                radius,
                half_side,
                directions,
            } => {
                let q = fold_to_direction(point - *center, directions);

                if q.y <= *radius {
                    q.y - radius
                } else {
                    TyVector2F64::new((q.x - half_side).max(0.0), q.y - radius).length()
                }
            }

            Shape2dField::Offset { shape_id, distance } => {
                shapes.distance2d(*shape_id, point) - distance
            }

            Shape2dField::Outline { contours } => outline_distance(contours, point),

            Shape2dField::Polygon { points } => polygon_distance(points, point),

            Shape2dField::Polyline { points, half_width } => {
                polyline_distance(points, *half_width, point)
            }

            Shape2dField::Rect { min, max, round } => {
                let half_extents = (*max - *min) / 2.0;
                rect_distance(point - (*min + *max) / 2.0, half_extents - *round) - round
            }

            Shape2dField::Sector {
                center,
                radius,
                span,
                ..
            } => sector_distance(point - *center, *radius, span),

            Shape2dField::Shell {
                shape_id,
                thickness,
            } => {
                let distance = shapes.distance2d(*shape_id, point);
                distance.max(-distance - thickness)
            }

            Shape2dField::SmoothIntersect { radius, shape_ids } => shape_ids
                .iter()
                .map(|shape_id| shapes.distance2d(*shape_id, point))
                .reduce(|a, b| smooth_intersect_distance(a, b, *radius))
                .expect("a smooth intersect holds a shape"),

            Shape2dField::SmoothSubtract {
                radius,
                base_id,
                cutter_ids,
            } => {
                cutter_ids
                    .iter()
                    .fold(shapes.distance2d(*base_id, point), |distance, cutter_id| {
                        smooth_intersect_distance(
                            distance,
                            -shapes.distance2d(*cutter_id, point),
                            *radius,
                        )
                    })
            }

            Shape2dField::SmoothUnion { radius, shape_ids } => shape_ids
                .iter()
                .map(|shape_id| shapes.distance2d(*shape_id, point))
                .reduce(|a, b| smooth_union_distance(a, b, *radius))
                .expect("a smooth union holds a shape"),

            Shape2dField::Star {
                center,
                outer_radius,
                inner_vertex,
                directions,
                ..
            } => {
                let q = fold_to_direction(point - *center, directions);
                let tip = TyVector2F64::new(0.0, *outer_radius);
                let e = *inner_vertex - tip;
                let w = q - tip;
                let distance = (w - e * (w.dot(e) / e.dot(e)).clamp(0.0, 1.0)).length();

                // The center's side of the edge holds the inside.
                if e.x * w.y - e.y * w.x < 0.0 {
                    -distance
                } else {
                    distance
                }
            }

            Shape2dField::Subtract {
                base_id,
                cutter_ids,
            } => cutter_ids
                .iter()
                .fold(shapes.distance2d(*base_id, point), |distance, cutter_id| {
                    distance.max(-shapes.distance2d(*cutter_id, point))
                }),

            Shape2dField::Transform {
                shape_id,
                map,
                distance_factor,
            } => distance_factor * shapes.distance2d(*shape_id, map.child_point(point)),

            Shape2dField::Union { shape_ids } => shape_ids
                .iter()
                .map(|shape_id| shapes.distance2d(*shape_id, point))
                .reduce(f64::min)
                .expect("a union holds a shape"),

            Shape2dField::Vesica { a, b, half_width } => {
                vesica_distance(point, *a, *b, *half_width)
            }
        }
    }

    /// The box around the shape. `bounds` holds the boxes of the shapes the
    /// field references.
    pub fn bounds(&self, bounds: &IdVec<BSdfShape2d, Bounds2d>) -> Bounds2d {
        let of = |shape_id: &U32Id<BSdfShape2d>| bounds[shape_id.to_usize_id()];

        match self {
            Shape2dField::Arc {
                center,
                radius,
                half_width,
                from_degrees,
                to_degrees,
                ..
            } => arc_bounds(*center, *radius, *from_degrees, *to_degrees)
                .grow(TyVector2F64::splat(*half_width)),

            Shape2dField::Arch { min, max }
            | Shape2dField::ChamferRect { min, max, .. }
            | Shape2dField::Rect { min, max, .. } => Bounds2d {
                min: *min,
                max: *max,
            },

            Shape2dField::Circle { center, radius } => {
                Bounds2d::around(*center, TyVector2F64::splat(*radius))
            }

            Shape2dField::Copies { shape_id, maps } => maps
                .iter()
                .map(|map| {
                    Bounds2d::from_points(
                        of(shape_id)
                            .corners()
                            .map(|corner| map.parent_point(corner)),
                    )
                })
                .reduce(|bounds, copy| bounds.union(&copy))
                .expect("a shape has at least one copy"),

            Shape2dField::Ellipse { center, radii } => Bounds2d::around(*center, *radii),

            Shape2dField::Intersect { shape_ids }
            | Shape2dField::SmoothIntersect { shape_ids, .. } => shape_ids
                .iter()
                .map(of)
                .reduce(|bounds, shape| bounds.intersection(&shape))
                .expect("an intersect holds a shape"),

            Shape2dField::Ngon {
                center,
                radius,
                half_side,
                directions,
            } => Bounds2d::from_points(directions.iter().map(|direction| {
                *center
                    + *direction * *radius
                    + TyVector2F64::new(-direction.y, direction.x) * *half_side
            })),

            Shape2dField::Offset { shape_id, distance } => {
                of(shape_id).grow(TyVector2F64::splat(distance.max(0.0)))
            }

            Shape2dField::Outline { contours } => contours
                .iter()
                .map(|contour| contour.bounds)
                .reduce(|bounds, contour_bounds| bounds.union(&contour_bounds))
                .expect("an outline holds contours"),

            Shape2dField::Polygon { points } => Bounds2d::from_points(points.iter().copied()),

            Shape2dField::Polyline { points, half_width } => {
                Bounds2d::from_points(points.iter().copied()).grow(TyVector2F64::splat(*half_width))
            }

            Shape2dField::Sector {
                center,
                radius,
                from_degrees,
                to_degrees,
                ..
            } => arc_bounds(*center, *radius, *from_degrees, *to_degrees).union(&Bounds2d {
                min: *center,
                max: *center,
            }),

            Shape2dField::Shell { shape_id, .. } => of(shape_id),

            Shape2dField::SmoothSubtract { base_id, .. }
            | Shape2dField::Subtract { base_id, .. } => of(base_id),

            Shape2dField::SmoothUnion { radius, shape_ids } => shape_ids
                .iter()
                .map(of)
                .reduce(|bounds, shape| {
                    bounds.union(&shape).grow(TyVector2F64::splat(radius / 4.0))
                })
                .expect("a smooth union holds a shape"),

            Shape2dField::Star {
                center,
                outer_radius,
                inner_radius,
                directions,
                ..
            } => {
                // Each inner vertex lies halfway between two tips.
                let points = directions.len() as f64;

                Bounds2d::from_points(directions.iter().enumerate().flat_map(|(index, tip)| {
                    let (sin, cos) = exact_sin_cos(90.0 + (index as f64 + 0.5) * 360.0 / points);

                    [
                        *center + *tip * *outer_radius,
                        *center + TyVector2F64::new(cos, sin) * *inner_radius,
                    ]
                }))
            }

            Shape2dField::Transform { shape_id, map, .. } => Bounds2d::from_points(
                of(shape_id)
                    .corners()
                    .map(|corner| map.parent_point(corner)),
            ),

            Shape2dField::Union { shape_ids } => shape_ids
                .iter()
                .map(of)
                .reduce(|bounds, shape| bounds.union(&shape))
                .expect("a union holds a shape"),

            Shape2dField::Vesica { a, b, half_width } => {
                Bounds2d::from_points([*a, *b]).grow(TyVector2F64::splat(*half_width))
            }
        }
    }
}

fn directions(first_degrees: f64, count: u32) -> Vec<TyVector2F64> {
    (0..count)
        .map(|index| {
            let (sin, cos) =
                exact_sin_cos(first_degrees + f64::from(index) * 360.0 / f64::from(count));
            TyVector2F64::new(cos, sin)
        })
        .collect()
}

/// The maps of a `mirror` in binary order with u as the low bit. The order
/// decides which copy wins a tie.
fn mirror_maps(axes: SdfAxes2d, center: TyVector2F64) -> Vec<PointMap2d> {
    let flags: u32 = match axes {
        SdfAxes2d::U => 0b01,
        SdfAxes2d::Uv => 0b11,
        SdfAxes2d::V => 0b10,
    };

    (0..4_u32)
        .filter(|mask| mask & !flags == 0)
        .map(|mask| PointMap2d::Reflection {
            axes: [mask & 0b01 != 0, mask & 0b10 != 0],
            center,
        })
        .collect()
}

fn arc_bounds(center: TyVector2F64, radius: f64, from_degrees: f64, to_degrees: f64) -> Bounds2d {
    let point_at = |degrees: f64| {
        let (sin, cos) = exact_sin_cos(degrees);
        center + TyVector2F64::new(cos, sin) * radius
    };

    let first_quarter = (from_degrees / 90.0).ceil() as i64;
    let last_quarter = (to_degrees / 90.0).floor() as i64;

    Bounds2d::from_points(
        [point_at(from_degrees), point_at(to_degrees)]
            .into_iter()
            .chain((first_quarter..=last_quarter).map(|quarter| point_at(quarter as f64 * 90.0))),
    )
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        assert_shape2d, grid2d, polygon_reference, rect_reference, shapes_of,
    };
    use branded_id::U32Id;
    use sdfcore::{BSdfShape2d, SdfAxes2d, SdfShape2d};
    use std::f64::consts::{PI, TAU};
    use ty_math::TyVector2F64;

    const GRID: (TyVector2F64, TyVector2F64, u32) =
        (TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17);

    fn circle(u: f64, v: f64, radius: f64) -> SdfShape2d {
        SdfShape2d::Circle {
            center: TyVector2F64::new(u, v),
            radius,
        }
    }

    fn id(index: u32) -> U32Id<BSdfShape2d> {
        U32Id::from_u32(index)
    }

    /// The point at `radius` from `center`, `turns` of a full turn
    /// counterclockwise from +u.
    fn polar(center: TyVector2F64, radius: f64, turns: f64) -> TyVector2F64 {
        let (sin, cos) = (turns * TAU).sin_cos();
        center + TyVector2F64::new(cos, sin) * radius
    }

    #[test]
    fn a_circle_measures_exact_distances() {
        assert_shape2d(vec![circle(0.1, -0.2, 0.6)], GRID, 1e-15, |point| {
            point.distance(TyVector2F64::new(0.1, -0.2)) - 0.6
        });
    }

    #[test]
    fn an_ngon_faces_one_side_down_at_its_radius() {
        let center = TyVector2F64::new(0.1, -0.2);
        let radius = 0.5;

        for sides in [3, 4, 5, 8] {
            let corner_radius = radius / (PI / f64::from(sides)).cos();
            let corners: Vec<TyVector2F64> = (0..sides)
                .map(|index| {
                    polar(
                        center,
                        corner_radius,
                        0.75 + (f64::from(index) + 0.5) / f64::from(sides),
                    )
                })
                .collect();

            let shape = SdfShape2d::Ngon {
                center,
                sides: f64::from(sides),
                radius,
            };

            assert_shape2d(vec![shape], GRID, 1e-12, |point| {
                polygon_reference(&corners, point)
            });
        }
    }

    #[test]
    fn a_square_ngon_spans_its_center_plus_and_minus_its_radius() {
        let shape = SdfShape2d::Ngon {
            center: TyVector2F64::ZERO,
            sides: 4.0,
            radius: 0.5,
        };

        assert_shape2d(vec![shape], GRID, 0.0, |point| {
            rect_reference(point, TyVector2F64::splat(-0.5), TyVector2F64::splat(0.5))
        });
    }

    #[test]
    fn a_star_puts_a_point_up() {
        let center = TyVector2F64::new(0.1, -0.2);

        for (points, outer_radius, inner_radius) in [(5, 0.7, 0.3), (2, 0.6, 0.2), (6, 0.5, 0.4)] {
            let outline: Vec<TyVector2F64> = (0..2 * points)
                .map(|index| {
                    let radius = if index % 2 == 0 {
                        outer_radius
                    } else {
                        inner_radius
                    };
                    polar(
                        center,
                        radius,
                        0.25 + f64::from(index) / f64::from(2 * points),
                    )
                })
                .collect();

            let shape = SdfShape2d::Star {
                center,
                points: f64::from(points),
                outer_radius,
                inner_radius,
            };

            assert_shape2d(vec![shape], GRID, 1e-12, |point| {
                polygon_reference(&outline, point)
            });
        }
    }

    #[test]
    fn booleans_fold_their_operands_left() {
        let a = |point: TyVector2F64| point.distance(TyVector2F64::new(-0.2, 0.0)) - 0.4;
        let b = |point: TyVector2F64| point.distance(TyVector2F64::new(0.3, 0.1)) - 0.3;
        let operands = || vec![circle(-0.2, 0.0, 0.4), circle(0.3, 0.1, 0.3)];
        let both = vec![id(0), id(1)];

        let with = |shape: SdfShape2d| {
            let mut shapes = operands();
            shapes.push(shape);
            shapes
        };

        assert_shape2d(
            with(SdfShape2d::Union {
                shape_ids: both.clone(),
            }),
            GRID,
            1e-15,
            |point| a(point).min(b(point)),
        );
        assert_shape2d(
            with(SdfShape2d::Intersect {
                shape_ids: both.clone(),
            }),
            GRID,
            1e-15,
            |point| a(point).max(b(point)),
        );
        assert_shape2d(
            with(SdfShape2d::Subtract {
                base_id: id(0),
                cutter_ids: vec![id(1)],
            }),
            GRID,
            1e-15,
            |point| a(point).max(-b(point)),
        );
        assert_shape2d(
            with(SdfShape2d::SmoothUnion {
                radius: 0.2,
                shape_ids: both.clone(),
            }),
            GRID,
            1e-15,
            |point| {
                let h = (0.2 - (a(point) - b(point)).abs()).max(0.0) / 0.2;
                a(point).min(b(point)) - h * h * 0.2 / 4.0
            },
        );
        assert_shape2d(
            with(SdfShape2d::SmoothSubtract {
                radius: 0.2,
                base_id: id(0),
                cutter_ids: vec![id(1)],
            }),
            GRID,
            1e-15,
            |point| {
                let (a, b) = (a(point), -b(point));
                let h = (0.2 - (a - b).abs()).max(0.0) / 0.2;
                a.max(b) + h * h * 0.2 / 4.0
            },
        );
    }

    #[test]
    fn a_quarter_turn_moves_a_rect_exactly() {
        let shapes = shapes_of(
            vec![
                SdfShape2d::Rect {
                    min: TyVector2F64::new(0.0, 0.0),
                    max: TyVector2F64::new(0.5, 0.25),
                    chamfer: None,
                    round: None,
                },
                SdfShape2d::Rotate {
                    shape_id: id(0),
                    degrees: 90.0,
                    pivot: None,
                },
                SdfShape2d::Rect {
                    min: TyVector2F64::new(-0.25, 0.0),
                    max: TyVector2F64::new(0.0, 0.5),
                    chamfer: None,
                    round: None,
                },
            ],
            Vec::new(),
        );

        for point in grid2d(TyVector2F64::splat(-1.0), TyVector2F64::splat(1.0), 17) {
            assert_eq!(
                shapes.distance2d(id(1), point),
                shapes.distance2d(id(2), point)
            );
        }
    }

    #[test]
    fn transforms_move_turn_and_scale_about_their_pivot() {
        let base = || circle(0.2, 0.0, 0.3);
        let pivot = TyVector2F64::new(0.1, 0.1);

        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Translate {
                    shape_id: id(0),
                    offset: TyVector2F64::new(-0.3, 0.2),
                },
            ],
            GRID,
            1e-15,
            |point| point.distance(TyVector2F64::new(-0.1, 0.2)) - 0.3,
        );
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Rotate {
                    shape_id: id(0),
                    degrees: 30.0,
                    pivot: Some(pivot),
                },
            ],
            GRID,
            1e-15,
            |point| {
                let (sin, cos) = 30.0_f64.to_radians().sin_cos();
                let offset = TyVector2F64::new(0.1, -0.1);
                let center = pivot
                    + TyVector2F64::new(
                        offset.x * cos - offset.y * sin,
                        offset.x * sin + offset.y * cos,
                    );
                point.distance(center) - 0.3
            },
        );
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Scale {
                    shape_id: id(0),
                    factor: TyVector2F64::splat(2.0),
                    pivot: Some(pivot),
                },
            ],
            GRID,
            1e-15,
            |point| point.distance(TyVector2F64::new(0.3, -0.1)) - 0.6,
        );
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Scale {
                    shape_id: id(0),
                    factor: TyVector2F64::new(2.0, 0.5),
                    pivot: None,
                },
            ],
            GRID,
            1e-15,
            |point| {
                0.5 * ((point / TyVector2F64::new(2.0, 0.5)).distance(TyVector2F64::new(0.2, 0.0))
                    - 0.3)
            },
        );
    }

    #[test]
    fn copies_union_every_place() {
        let base = || circle(0.3, 0.2, 0.15);
        let nearest = |centers: &[TyVector2F64], point: TyVector2F64| {
            centers
                .iter()
                .map(|center| point.distance(*center) - 0.15)
                .fold(f64::INFINITY, f64::min)
        };

        let mirrored = [(0.3, 0.2), (-0.3, 0.2), (0.3, -0.2), (-0.3, -0.2)]
            .map(|(u, v)| TyVector2F64::new(u, v));
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Mirror {
                    shape_id: id(0),
                    axes: SdfAxes2d::Uv,
                    center: None,
                },
            ],
            GRID,
            1e-15,
            |point| nearest(&mirrored, point),
        );

        let repeated =
            [(0.3, 0.2), (0.0, 0.2), (0.3, 0.5), (0.0, 0.5)].map(|(u, v)| TyVector2F64::new(u, v));
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::Repeat {
                    shape_id: id(0),
                    step: TyVector2F64::new(-0.3, 0.3),
                    count: TyVector2F64::new(2.0, 2.0),
                },
            ],
            GRID,
            1e-15,
            |point| nearest(&repeated, point),
        );

        let turned = [(0.3, 0.2), (-0.2, 0.3), (-0.3, -0.2), (0.2, -0.3)]
            .map(|(u, v)| TyVector2F64::new(u, v));
        assert_shape2d(
            vec![
                base(),
                SdfShape2d::RepeatPolar {
                    shape_id: id(0),
                    count: 4.0,
                    center: None,
                },
            ],
            GRID,
            1e-15,
            |point| nearest(&turned, point),
        );
    }

    #[test]
    fn offset_and_shell_move_the_surface() {
        let disk = |point: TyVector2F64| point.length() - 0.5;

        assert_shape2d(
            vec![
                circle(0.0, 0.0, 0.5),
                SdfShape2d::Offset {
                    shape_id: id(0),
                    distance: -0.2,
                },
            ],
            GRID,
            1e-15,
            |point| disk(point) + 0.2,
        );
        assert_shape2d(
            vec![
                circle(0.0, 0.0, 0.5),
                SdfShape2d::Shell {
                    shape_id: id(0),
                    thickness: 0.1,
                },
            ],
            GRID,
            1e-15,
            |point| disk(point).max(-disk(point) - 0.1),
        );
    }
}
