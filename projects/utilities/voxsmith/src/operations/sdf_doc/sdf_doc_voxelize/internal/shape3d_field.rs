use crate::operations::sdf_doc::{
    ArcSpan, BendMap, Bounds2d, Bounds3d, PointMap3d, SdfEvaluation, ShapeSource, SweepSample,
    align_rotation, axis_rotation, box_distance, box_frame_distance, cone_distance,
    ellipsoid_distance, exact_sin_cos, fbm, noise_seed, octahedron_distance, plane_axes,
    polygon_distance, pyramid_distance, rect_distance, round_cone_distance, side_direction,
    smooth_intersect_distance, smooth_union_distance, whole_count,
};
use branded_id::{IdVec, U32Id};
use sdfcore::{BSdfShape2d, BSdfShape3d, SdfAxes3d, SdfShape3d};
use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

/// The octaves a `displace` sums when the model leaves them out.
const DEFAULT_OCTAVES: u32 = 4;

/// A 3D shape prepared for evaluation, with every value that holds for all
/// points computed once.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape3dField {
    /// A shape curled along one axis into an arc.
    Bend {
        shape_id: U32Id<BSdfShape3d>,

        map: BendMap,
    },

    /// A box with edges rounded by `round`, which reads 0 for square edges.
    Box {
        min: TyVector3F64,

        max: TyVector3F64,

        round: f64,
    },

    /// The twelve edges of a box.
    BoxFrame {
        min: TyVector3F64,

        max: TyVector3F64,

        half_thickness: f64,
    },

    /// A capsule between two points.
    Capsule {
        a: TyVector3F64,

        b: TyVector3F64,

        radius: f64,
    },

    /// A cone between two points with a radius at each.
    Cone {
        a: TyVector3F64,

        b: TyVector3F64,

        radius_a: f64,

        radius_b: f64,
    },

    /// The union of a shape's copies, one per map.
    Copies {
        shape_id: U32Id<BSdfShape3d>,

        maps: Vec<PointMap3d>,
    },

    /// A cylinder between two points, with rims rounded by `round`, which
    /// reads 0 for square rims.
    Cylinder {
        a: TyVector3F64,

        b: TyVector3F64,

        axis: TyVector3F64,

        radius: f64,

        round: f64,
    },

    /// A shape roughened with fractal noise.
    Displace {
        shape_id: U32Id<BSdfShape3d>,

        amplitude: f64,

        scale: f64,

        octaves: u32,

        seed: u32,
    },

    /// A shape cut through `center` with its halves moved apart.
    Elongate {
        shape_id: U32Id<BSdfShape3d>,

        center: TyVector3F64,

        half_lengths: TyVector3F64,
    },

    /// An ellipsoid.
    Ellipsoid {
        center: TyVector3F64,

        radii: TyVector3F64,
    },

    /// A profile pushed along the axis at index `axis`. The profile's u and v
    /// map to the axes at `plane`.
    Extrude {
        profile_id: U32Id<BSdfShape2d>,

        axis: usize,

        plane: [usize; 2],

        from: f64,

        to: f64,
    },

    /// Everything past `at` along the axis at index `axis`, toward the axis's
    /// positive end when `positive` holds.
    HalfSpace {
        axis: usize,

        positive: bool,

        at: f64,
    },

    /// The cells every shape covers.
    Intersect { shape_ids: Vec<U32Id<BSdfShape3d>> },

    /// The outline of a `lathe` and its mirror image across the axis, revolved
    /// about the axis at index `axis` through `center`.
    Lathe {
        outline: Vec<TyVector2F64>,

        axis: usize,

        plane: [usize; 2],

        center: TyVector3F64,
    },

    /// An octahedron.
    Octahedron { center: TyVector3F64, radius: f64 },

    /// A shape grown or shrunk.
    Offset {
        shape_id: U32Id<BSdfShape3d>,

        distance: f64,
    },

    /// A square pyramid.
    Pyramid {
        base_center: TyVector3F64,

        width: f64,

        height: f64,
    },

    /// A profile revolved about the axis at index `axis` through `center`.
    Revolve {
        profile_id: U32Id<BSdfShape2d>,

        axis: usize,

        plane: [usize; 2],

        center: TyVector3F64,
    },

    /// A cone with rounded ends, whose unit axis runs from `a` to `b`.
    RoundCone {
        a: TyVector3F64,

        b: TyVector3F64,

        axis: TyVector3F64,

        radius_a: f64,

        radius_b: f64,
    },

    /// A shape's outer wall.
    Shell {
        shape_id: U32Id<BSdfShape3d>,

        thickness: f64,
    },

    /// An intersection with rounded edges.
    SmoothIntersect {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A subtraction with rounded edges.
    SmoothSubtract {
        radius: f64,

        base_id: U32Id<BSdfShape3d>,

        cutter_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A union with filleted corners.
    SmoothUnion {
        radius: f64,

        shape_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A sphere.
    Sphere { center: TyVector3F64, radius: f64 },

    /// A base shape with the cutters removed.
    Subtract {
        base_id: U32Id<BSdfShape3d>,

        cutter_ids: Vec<U32Id<BSdfShape3d>>,
    },

    /// A profile pushed along the curve its samples trace, with the profile's
    /// v kept as near the unit `up` as the curve allows.
    Sweep {
        profile_id: U32Id<BSdfShape2d>,

        samples: Vec<SweepSample>,

        up: TyVector3F64,
    },

    /// A torus about the axis at index `axis`, cut to `span` when the span
    /// holds one.
    Torus {
        center: TyVector3F64,

        axis: usize,

        plane: [usize; 2],

        ring_radius: f64,

        tube_radius: f64,

        span: Option<ArcSpan>,
    },

    /// A shape moved, turned, or scaled, whose distance scales by
    /// `distance_factor`.
    Transform {
        shape_id: U32Id<BSdfShape3d>,

        map: PointMap3d,

        distance_factor: f64,
    },

    /// A shape twisted about an axis.
    Twist {
        shape_id: U32Id<BSdfShape3d>,

        axis: TyAxis3,

        degrees_per_meter: f64,

        center: TyVector3F64,
    },

    /// The cells any shape covers.
    Union { shape_ids: Vec<U32Id<BSdfShape3d>> },
}

impl Shape3dField {
    /// The field of `shape`. `bounds` holds the boxes of the shapes `shape`
    /// references, which a `bend` reads.
    pub fn new(shape: &SdfShape3d, bounds: &IdVec<BSdfShape3d, Option<Bounds3d>>) -> Self {
        match shape {
            SdfShape3d::Bend {
                shape_id,
                along,
                toward,
                radius,
                pivot,
            } => Shape3dField::Bend {
                shape_id: *shape_id,
                map: BendMap::new(
                    *along,
                    *toward,
                    *radius,
                    pivot.unwrap_or(TyVector3F64::ZERO),
                    &bounds[shape_id.to_usize_id()].expect("a bend's shape has a box"),
                ),
            },

            // A box takes its corners in either order on each axis.
            SdfShape3d::Box { min, max, round } => Shape3dField::Box {
                min: min.min(*max),
                max: min.max(*max),
                round: round.unwrap_or(0.0),
            },

            SdfShape3d::BoxFrame {
                min,
                max,
                thickness,
            } => Shape3dField::BoxFrame {
                min: min.min(*max),
                max: min.max(*max),
                half_thickness: thickness / 2.0,
            },

            SdfShape3d::Capsule { a, b, radius } => Shape3dField::Capsule {
                a: *a,
                b: *b,
                radius: *radius,
            },

            SdfShape3d::Cone {
                a,
                b,
                radius_a,
                radius_b,
            } => Shape3dField::Cone {
                a: *a,
                b: *b,
                radius_a: *radius_a,
                radius_b: *radius_b,
            },

            SdfShape3d::Cylinder {
                a,
                b,
                radius,
                round,
            } => Shape3dField::Cylinder {
                a: *a,
                b: *b,
                axis: unit(*b - *a),
                radius: *radius,
                round: round.unwrap_or(0.0),
            },

            SdfShape3d::Displace {
                shape_id,
                amplitude,
                scale,
                octaves,
                seed,
            } => Shape3dField::Displace {
                shape_id: *shape_id,
                amplitude: *amplitude,
                scale: *scale,
                octaves: octaves.map_or(DEFAULT_OCTAVES, whole_count),
                seed: noise_seed(*seed),
            },

            SdfShape3d::Elongate {
                shape_id,
                lengths,
                center,
            } => Shape3dField::Elongate {
                shape_id: *shape_id,
                center: center.unwrap_or(TyVector3F64::ZERO),
                half_lengths: *lengths / 2.0,
            },

            SdfShape3d::Ellipsoid { center, radii } => Shape3dField::Ellipsoid {
                center: *center,
                radii: *radii,
            },

            SdfShape3d::Extrude {
                profile_id,
                axis,
                from,
                to,
            } => {
                let axis = axis.unwrap_or(TyAxis3::Z);

                Shape3dField::Extrude {
                    profile_id: *profile_id,
                    axis: axis.index(),
                    plane: plane_axes(axis),
                    from: *from,
                    to: *to,
                }
            }

            SdfShape3d::HalfSpace { side, at } => {
                let (axis, sign) = side_direction(*side);

                Shape3dField::HalfSpace {
                    axis: axis.index(),
                    positive: sign > 0.0,
                    at: *at,
                }
            }

            SdfShape3d::Intersect { shape_ids } => Shape3dField::Intersect {
                shape_ids: shape_ids.clone(),
            },

            SdfShape3d::Lathe {
                points,
                axis,
                center,
            } => {
                let axis = axis.unwrap_or(TyAxis3::Y);

                Shape3dField::Lathe {
                    // The points and their mirror images across the axis close
                    // the outline at the first and last heights. No edge runs
                    // along the axis for a point near it to measure to.
                    outline: points
                        .iter()
                        .copied()
                        .chain(
                            points
                                .iter()
                                .rev()
                                .map(|point| TyVector2F64::new(-point.x, point.y)),
                        )
                        .collect(),
                    axis: axis.index(),
                    plane: plane_axes(axis),
                    center: center.unwrap_or(TyVector3F64::ZERO),
                }
            }

            SdfShape3d::Mirror {
                shape_id,
                axes,
                center,
            } => Shape3dField::Copies {
                shape_id: *shape_id,
                maps: mirror_maps(*axes, center.unwrap_or(TyVector3F64::ZERO)),
            },

            SdfShape3d::Octahedron { center, radius } => Shape3dField::Octahedron {
                center: *center,
                radius: *radius,
            },

            SdfShape3d::Offset { shape_id, distance } => Shape3dField::Offset {
                shape_id: *shape_id,
                distance: *distance,
            },

            SdfShape3d::Orient {
                shape_id,
                from,
                to,
                pivot,
            } => Shape3dField::Transform {
                shape_id: *shape_id,
                map: PointMap3d::Rotation {
                    rows: align_rotation(unit(*from), unit(*to)),
                    pivot: pivot.unwrap_or(TyVector3F64::ZERO),
                },
                distance_factor: 1.0,
            },

            SdfShape3d::Pyramid {
                base_center,
                width,
                height,
            } => Shape3dField::Pyramid {
                base_center: *base_center,
                width: *width,
                height: *height,
            },

            SdfShape3d::Repeat {
                shape_id,
                step,
                count,
            } => {
                let counts = [
                    whole_count(count.x),
                    whole_count(count.y),
                    whole_count(count.z),
                ];

                Shape3dField::Copies {
                    shape_id: *shape_id,
                    maps: (0..counts[0])
                        .flat_map(|i| (0..counts[1]).map(move |j| (i, j)))
                        .flat_map(|(i, j)| (0..counts[2]).map(move |k| (i, j, k)))
                        .map(|(i, j, k)| PointMap3d::Translation {
                            offset: TyVector3F64::new(f64::from(i), f64::from(j), f64::from(k))
                                * *step,
                        })
                        .collect(),
                }
            }

            SdfShape3d::RepeatPolar {
                shape_id,
                axis,
                count,
                center,
            } => {
                let count = whole_count(*count);
                let pivot = center.unwrap_or(TyVector3F64::ZERO);

                Shape3dField::Copies {
                    shape_id: *shape_id,
                    maps: (0..count)
                        .map(|index| {
                            let (sin, cos) =
                                exact_sin_cos(f64::from(index) * 360.0 / f64::from(count));

                            PointMap3d::Rotation {
                                rows: axis_rotation(*axis, sin, cos),
                                pivot,
                            }
                        })
                        .collect(),
                }
            }

            SdfShape3d::Revolve {
                profile_id,
                axis,
                center,
            } => {
                let axis = axis.unwrap_or(TyAxis3::Y);

                Shape3dField::Revolve {
                    profile_id: *profile_id,
                    axis: axis.index(),
                    plane: plane_axes(axis),
                    center: center.unwrap_or(TyVector3F64::ZERO),
                }
            }

            SdfShape3d::Rotate {
                shape_id,
                axis,
                degrees,
                pivot,
            } => {
                let (sin, cos) = exact_sin_cos(*degrees);

                Shape3dField::Transform {
                    shape_id: *shape_id,
                    map: PointMap3d::Rotation {
                        rows: axis_rotation(*axis, sin, cos),
                        pivot: pivot.unwrap_or(TyVector3F64::ZERO),
                    },
                    distance_factor: 1.0,
                }
            }

            SdfShape3d::RoundCone {
                a,
                b,
                radius_a,
                radius_b,
            } => Shape3dField::RoundCone {
                a: *a,
                b: *b,
                axis: unit(*b - *a),
                radius_a: *radius_a,
                radius_b: *radius_b,
            },

            SdfShape3d::Scale {
                shape_id,
                factor,
                pivot,
            } => Shape3dField::Transform {
                shape_id: *shape_id,
                map: PointMap3d::Scaling {
                    factor: *factor,
                    pivot: pivot.unwrap_or(TyVector3F64::ZERO),
                },
                distance_factor: if factor.x == factor.y && factor.y == factor.z {
                    factor.x
                } else {
                    factor.min_element()
                },
            },

            SdfShape3d::Shell {
                shape_id,
                thickness,
            } => Shape3dField::Shell {
                shape_id: *shape_id,
                thickness: *thickness,
            },

            SdfShape3d::SmoothIntersect { radius, shape_ids } => Shape3dField::SmoothIntersect {
                radius: *radius,
                shape_ids: shape_ids.clone(),
            },

            SdfShape3d::SmoothSubtract {
                radius,
                base_id,
                cutter_ids,
            } => Shape3dField::SmoothSubtract {
                radius: *radius,
                base_id: *base_id,
                cutter_ids: cutter_ids.clone(),
            },

            SdfShape3d::SmoothUnion { radius, shape_ids } => Shape3dField::SmoothUnion {
                radius: *radius,
                shape_ids: shape_ids.clone(),
            },

            SdfShape3d::Sphere { center, radius } => Shape3dField::Sphere {
                center: *center,
                radius: *radius,
            },

            SdfShape3d::Subtract {
                base_id,
                cutter_ids,
            } => Shape3dField::Subtract {
                base_id: *base_id,
                cutter_ids: cutter_ids.clone(),
            },

            SdfShape3d::Sweep {
                profile_id,
                points,
                scales,
                up,
            } => Shape3dField::Sweep {
                profile_id: *profile_id,
                samples: SweepSample::along(points, scales.as_deref()),
                up: unit(up.unwrap_or(TyVector3F64::Y)),
            },

            SdfShape3d::Torus {
                center,
                ring_radius,
                tube_radius,
                axis,
                from,
                to,
            } => {
                let axis = axis.unwrap_or(TyAxis3::Y);

                let span = match (from, to) {
                    (Some(from), Some(to)) => Some(ArcSpan::new(*from, *to)),
                    (None, None) => None,
                    _ => panic!("a torus takes both from and to or neither"),
                };

                Shape3dField::Torus {
                    center: *center,
                    axis: axis.index(),
                    plane: plane_axes(axis),
                    ring_radius: *ring_radius,
                    tube_radius: *tube_radius,
                    span,
                }
            }

            SdfShape3d::Translate { shape_id, offset } => Shape3dField::Transform {
                shape_id: *shape_id,
                map: PointMap3d::Translation { offset: *offset },
                distance_factor: 1.0,
            },

            SdfShape3d::Twist {
                shape_id,
                axis,
                degrees_per_meter,
                center,
            } => Shape3dField::Twist {
                shape_id: *shape_id,
                axis: *axis,
                degrees_per_meter: *degrees_per_meter,
                center: center.unwrap_or(TyVector3F64::ZERO),
            },

            SdfShape3d::Union { shape_ids } => Shape3dField::Union {
                shape_ids: shape_ids.clone(),
            },
        }
    }

    /// The shape's distance and frame position at `point`. `shapes` holds the
    /// shapes the field references.
    pub fn evaluate(&self, shapes: &impl ShapeSource, point: TyVector3F64) -> SdfEvaluation {
        match self {
            Shape3dField::Bend { shape_id, map } => {
                shapes.evaluate(*shape_id, map.child_point(point))
            }

            Shape3dField::Box { min, max, round } => {
                let half_extents = (*max - *min) / 2.0;
                let distance =
                    box_distance(point - (*min + *max) / 2.0, half_extents - *round) - round;

                primitive(distance, point)
            }

            Shape3dField::BoxFrame {
                min,
                max,
                half_thickness,
            } => primitive(
                box_frame_distance(
                    point - (*min + *max) / 2.0,
                    (*max - *min) / 2.0,
                    *half_thickness,
                ),
                point,
            ),

            Shape3dField::Capsule { a, b, radius } => {
                let pa = point - *a;
                let ba = *b - *a;
                let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);

                primitive((pa - ba * h).length() - radius, point)
            }

            Shape3dField::Cone {
                a,
                b,
                radius_a,
                radius_b,
            } => {
                let pa = point - *a;
                let ba = *b - *a;
                let length_squared = ba.dot(ba);
                let t = pa.dot(ba) / length_squared;
                let radial = (pa - ba * t).length();

                primitive(
                    cone_distance(radial, t, length_squared, *radius_a, *radius_b),
                    point,
                )
            }

            Shape3dField::Copies { shape_id, maps } => maps
                .iter()
                .map(|map| shapes.evaluate(*shape_id, map.child_point(point)))
                .reduce(nearer)
                .expect("a shape has at least one copy"),

            Shape3dField::Cylinder {
                a,
                b,
                axis,
                radius,
                round,
            } => {
                let w = point - (*a + *b) / 2.0;
                let along = w.dot(*axis);
                let radial = (w - *axis * along).length();
                let half_extents = TyVector2F64::new(*radius, (*b - *a).length() / 2.0);
                let distance =
                    rect_distance(TyVector2F64::new(radial, along), half_extents - *round) - round;

                primitive(distance, point)
            }

            Shape3dField::Displace {
                shape_id,
                amplitude,
                scale,
                octaves,
                seed,
            } => {
                let evaluation = shapes.evaluate(*shape_id, point);

                SdfEvaluation {
                    distance: evaluation.distance
                        + amplitude * fbm(point / *scale, *octaves, *seed),
                    ..evaluation
                }
            }

            Shape3dField::Elongate {
                shape_id,
                center,
                half_lengths,
            } => shapes.evaluate(
                *shape_id,
                point - (point - *center).clamp(-*half_lengths, *half_lengths),
            ),

            Shape3dField::Ellipsoid { center, radii } => {
                primitive(ellipsoid_distance(point - *center, *radii), point)
            }

            Shape3dField::Extrude {
                profile_id,
                axis,
                plane,
                from,
                to,
            } => {
                let profile = shapes.distance2d(
                    *profile_id,
                    TyVector2F64::new(point[plane[0]], point[plane[1]]),
                );
                let slab = (from - point[*axis]).max(point[*axis] - to);
                let distance = profile.max(slab).min(0.0)
                    + TyVector2F64::new(profile.max(0.0), slab.max(0.0)).length();

                primitive(distance, point)
            }

            Shape3dField::HalfSpace { axis, positive, at } => {
                let distance = if *positive {
                    at - point[*axis]
                } else {
                    point[*axis] - at
                };

                primitive(distance, point)
            }

            Shape3dField::Intersect { shape_ids } => {
                let mut evaluations = shape_ids
                    .iter()
                    .map(|shape_id| shapes.evaluate(*shape_id, point));
                let first = evaluations.next().expect("an intersect holds a shape");

                SdfEvaluation {
                    distance: evaluations.fold(first.distance, |distance, evaluation| {
                        distance.max(evaluation.distance)
                    }),
                    ..first
                }
            }

            Shape3dField::Lathe {
                outline,
                axis,
                plane,
                center,
            } => primitive(
                polygon_distance(outline, revolve_point(point, *axis, *plane, *center)),
                point,
            ),

            Shape3dField::Octahedron { center, radius } => {
                primitive(octahedron_distance(point - *center, *radius), point)
            }

            Shape3dField::Offset { shape_id, distance } => {
                let evaluation = shapes.evaluate(*shape_id, point);

                SdfEvaluation {
                    distance: evaluation.distance - distance,
                    ..evaluation
                }
            }

            Shape3dField::Pyramid {
                base_center,
                width,
                height,
            } => primitive(
                width * pyramid_distance((point - *base_center) / *width, height / width),
                point,
            ),

            Shape3dField::Revolve {
                profile_id,
                axis,
                plane,
                center,
            } => primitive(
                shapes.distance2d(*profile_id, revolve_point(point, *axis, *plane, *center)),
                point,
            ),

            Shape3dField::RoundCone {
                a,
                b,
                axis,
                radius_a,
                radius_b,
            } => {
                let pa = point - *a;
                let along = pa.dot(*axis);
                let radial = (pa - *axis * along).length();

                primitive(
                    round_cone_distance(radial, along, *radius_a, *radius_b, (*b - *a).length()),
                    point,
                )
            }

            Shape3dField::Shell {
                shape_id,
                thickness,
            } => {
                let evaluation = shapes.evaluate(*shape_id, point);

                SdfEvaluation {
                    distance: evaluation.distance.max(-evaluation.distance - thickness),
                    ..evaluation
                }
            }

            Shape3dField::SmoothIntersect { radius, shape_ids } => {
                let mut evaluations = shape_ids
                    .iter()
                    .map(|shape_id| shapes.evaluate(*shape_id, point));
                let first = evaluations
                    .next()
                    .expect("a smooth intersect holds a shape");

                SdfEvaluation {
                    distance: evaluations.fold(first.distance, |distance, evaluation| {
                        smooth_intersect_distance(distance, evaluation.distance, *radius)
                    }),
                    ..first
                }
            }

            Shape3dField::SmoothSubtract {
                radius,
                base_id,
                cutter_ids,
            } => {
                let base = shapes.evaluate(*base_id, point);

                SdfEvaluation {
                    distance: cutter_ids
                        .iter()
                        .fold(base.distance, |distance, cutter_id| {
                            smooth_intersect_distance(
                                distance,
                                -shapes.evaluate(*cutter_id, point).distance,
                                *radius,
                            )
                        }),
                    ..base
                }
            }

            Shape3dField::SmoothUnion { radius, shape_ids } => {
                let mut evaluations = shape_ids
                    .iter()
                    .map(|shape_id| shapes.evaluate(*shape_id, point));
                let first = evaluations.next().expect("a smooth union holds a shape");

                let (distance, nearest) = evaluations.fold(
                    (first.distance, first),
                    |(distance, nearest), evaluation| {
                        (
                            smooth_union_distance(distance, evaluation.distance, *radius),
                            nearer(nearest, evaluation),
                        )
                    },
                );

                SdfEvaluation {
                    distance,
                    ..nearest
                }
            }

            Shape3dField::Sphere { center, radius } => {
                primitive((point - *center).length() - radius, point)
            }

            Shape3dField::Subtract {
                base_id,
                cutter_ids,
            } => {
                let base = shapes.evaluate(*base_id, point);

                SdfEvaluation {
                    distance: cutter_ids
                        .iter()
                        .fold(base.distance, |distance, cutter_id| {
                            distance.max(-shapes.evaluate(*cutter_id, point).distance)
                        }),
                    ..base
                }
            }

            Shape3dField::Sweep {
                profile_id,
                samples,
                up,
            } => {
                let (index, fraction) = samples
                    .windows(2)
                    .enumerate()
                    .map(|(index, pair)| {
                        let start = pair[0].position;
                        let along = pair[1].position - start;
                        let fraction =
                            ((point - start).dot(along) / along.dot(along)).clamp(0.0, 1.0);

                        (
                            index,
                            fraction,
                            (point - start - along * fraction).length_squared(),
                        )
                    })
                    .min_by(|a, b| a.2.total_cmp(&b.2))
                    .map(|(index, fraction, _)| (index, fraction))
                    .expect("a sweep's curve holds a piece");
                let [start, end] = [samples[index], samples[index + 1]];
                let offset = point - start.position.lerp(end.position, fraction);
                let tangent = start.tangent.lerp(end.tangent, fraction).normalize();
                let scale = start.scale + (end.scale - start.scale) * fraction;
                let u = tangent.cross(*up).normalize();
                let local = TyVector2F64::new(offset.dot(u), offset.dot(u.cross(tangent)));
                let profile = if scale > 0.0 {
                    scale * shapes.distance2d(*profile_id, local / scale)
                } else {
                    local.length()
                };

                // Only the curve's two ends cap the profile.
                let distance = if (index == 0 && fraction == 0.0)
                    || (index == samples.len() - 2 && fraction == 1.0)
                {
                    let slab = offset.dot(tangent).abs();
                    profile.max(slab).min(0.0) + TyVector2F64::new(profile.max(0.0), slab).length()
                } else {
                    profile
                };

                primitive(distance, point)
            }

            Shape3dField::Torus {
                center,
                axis,
                plane,
                ring_radius,
                tube_radius,
                span,
            } => {
                let offset = point - *center;
                let along = offset[*axis];
                let across = TyVector2F64::new(offset[plane[0]], offset[plane[1]]);

                let to_ring = match span {
                    Some(span) => {
                        let folded = span.fold(across);

                        if span.is_past_end(folded) {
                            let end = folded - span.end(*ring_radius);
                            TyVector3F64::new(end.x, end.y, along).length()
                        } else {
                            TyVector2F64::new(folded.length() - ring_radius, along).length()
                        }
                    }

                    None => TyVector2F64::new(across.length() - ring_radius, along).length(),
                };

                primitive(to_ring - tube_radius, point)
            }

            Shape3dField::Transform {
                shape_id,
                map,
                distance_factor,
            } => {
                let evaluation = shapes.evaluate(*shape_id, map.child_point(point));

                SdfEvaluation {
                    distance: distance_factor * evaluation.distance,
                    ..evaluation
                }
            }

            Shape3dField::Twist {
                shape_id,
                axis,
                degrees_per_meter,
                center,
            } => {
                let degrees = -degrees_per_meter * (point[axis.index()] - center[axis.index()]);
                let (sin, cos) = degrees.to_radians().sin_cos();
                let rows = axis_rotation(*axis, sin, cos);
                let offset = point - *center;
                let turned = TyVector3F64::new(
                    rows[0].dot(offset),
                    rows[1].dot(offset),
                    rows[2].dot(offset),
                );

                shapes.evaluate(*shape_id, turned + *center)
            }

            Shape3dField::Union { shape_ids } => shape_ids
                .iter()
                .map(|shape_id| shapes.evaluate(*shape_id, point))
                .reduce(nearer)
                .expect("a union holds a shape"),
        }
    }

    /// The box around the shape, or `None` for a shape that reaches without
    /// end. `bounds3d` and `bounds2d` hold the boxes of the shapes the field
    /// references.
    pub fn bounds(
        &self,
        bounds3d: &IdVec<BSdfShape3d, Option<Bounds3d>>,
        bounds2d: &IdVec<BSdfShape2d, Bounds2d>,
    ) -> Option<Bounds3d> {
        let of = |shape_id: &U32Id<BSdfShape3d>| bounds3d[shape_id.to_usize_id()];
        let of_profile = |profile_id: &U32Id<BSdfShape2d>| bounds2d[profile_id.to_usize_id()];

        match self {
            Shape3dField::Bend { shape_id, map } => of(shape_id).map(|child| map.bounds(&child)),

            Shape3dField::Box { min, max, .. } | Shape3dField::BoxFrame { min, max, .. } => {
                Some(Bounds3d {
                    min: *min,
                    max: *max,
                })
            }

            Shape3dField::Capsule { a, b, radius } => {
                Some(Bounds3d::from_points([*a, *b]).grow(TyVector3F64::splat(*radius)))
            }

            Shape3dField::Cone {
                a,
                b,
                radius_a,
                radius_b,
            } => {
                let axis = unit(*b - *a);
                Some(disk_bounds(*a, axis, *radius_a).union(&disk_bounds(*b, axis, *radius_b)))
            }

            Shape3dField::Copies { shape_id, maps } => of(shape_id).map(|child| {
                maps.iter()
                    .map(|map| {
                        Bounds3d::from_points(
                            child.corners().map(|corner| map.parent_point(corner)),
                        )
                    })
                    .reduce(|bounds, copy| bounds.union(&copy))
                    .expect("a shape has at least one copy")
            }),

            Shape3dField::Cylinder {
                a, b, axis, radius, ..
            } => Some(disk_bounds(*a, *axis, *radius).union(&disk_bounds(*b, *axis, *radius))),

            Shape3dField::Displace {
                shape_id,
                amplitude,
                ..
            } => of(shape_id).map(|child| child.grow(TyVector3F64::splat(*amplitude))),

            Shape3dField::Elongate {
                shape_id,
                half_lengths,
                ..
            } => of(shape_id).map(|child| child.grow(*half_lengths)),

            Shape3dField::Ellipsoid { center, radii } => Some(Bounds3d::around(*center, *radii)),

            Shape3dField::Extrude {
                profile_id,
                axis,
                plane,
                from,
                to,
            } => {
                let profile = of_profile(profile_id);
                let mut bounds = Bounds3d {
                    min: TyVector3F64::ZERO,
                    max: TyVector3F64::ZERO,
                };

                bounds.min[*axis] = *from;
                bounds.max[*axis] = *to;
                bounds.min[plane[0]] = profile.min.x;
                bounds.max[plane[0]] = profile.max.x;
                bounds.min[plane[1]] = profile.min.y;
                bounds.max[plane[1]] = profile.max.y;

                Some(bounds)
            }

            Shape3dField::HalfSpace { .. } => None,

            Shape3dField::Intersect { shape_ids }
            | Shape3dField::SmoothIntersect { shape_ids, .. } => shape_ids
                .iter()
                .filter_map(of)
                .reduce(|bounds, shape| bounds.intersection(&shape)),

            Shape3dField::Lathe {
                outline,
                axis,
                center,
                ..
            } => Some(revolve_bounds(
                &Bounds2d::from_points(outline.iter().copied()),
                *axis,
                *center,
            )),

            Shape3dField::Octahedron { center, radius }
            | Shape3dField::Sphere { center, radius } => {
                Some(Bounds3d::around(*center, TyVector3F64::splat(*radius)))
            }

            Shape3dField::Offset { shape_id, distance } => {
                of(shape_id).map(|child| child.grow(TyVector3F64::splat(distance.max(0.0))))
            }

            Shape3dField::Pyramid {
                base_center,
                width,
                height,
            } => Some(Bounds3d {
                min: *base_center - TyVector3F64::new(width / 2.0, 0.0, width / 2.0),
                max: *base_center + TyVector3F64::new(width / 2.0, *height, width / 2.0),
            }),

            Shape3dField::Revolve {
                profile_id,
                axis,
                center,
                ..
            } => Some(revolve_bounds(&of_profile(profile_id), *axis, *center)),

            Shape3dField::RoundCone {
                a,
                b,
                radius_a,
                radius_b,
                ..
            } => Some(
                Bounds3d::around(*a, TyVector3F64::splat(*radius_a))
                    .union(&Bounds3d::around(*b, TyVector3F64::splat(*radius_b))),
            ),

            Shape3dField::Shell { shape_id, .. } => of(shape_id),

            Shape3dField::SmoothSubtract { base_id, .. }
            | Shape3dField::Subtract { base_id, .. } => of(base_id),

            Shape3dField::Sweep {
                profile_id,
                samples,
                ..
            } => {
                let profile = of_profile(profile_id);
                let reach = profile.min.abs().max(profile.max.abs()).length();

                samples
                    .iter()
                    .map(|sample| {
                        Bounds3d::around(sample.position, TyVector3F64::splat(reach * sample.scale))
                    })
                    .reduce(|bounds, sample| bounds.union(&sample))
            }

            Shape3dField::SmoothUnion { radius, shape_ids } => shape_ids
                .iter()
                .map(of)
                .reduce(|bounds, shape| {
                    bounds.zip(shape).map(|(bounds, shape)| {
                        bounds.union(&shape).grow(TyVector3F64::splat(radius / 4.0))
                    })
                })
                .expect("a smooth union holds a shape"),

            Shape3dField::Torus {
                center,
                axis,
                ring_radius,
                tube_radius,
                ..
            } => {
                let mut half_extents = TyVector3F64::splat(ring_radius + tube_radius);
                half_extents[*axis] = *tube_radius;

                Some(Bounds3d::around(*center, half_extents))
            }

            Shape3dField::Transform { shape_id, map, .. } => of(shape_id).map(|child| {
                Bounds3d::from_points(child.corners().map(|corner| map.parent_point(corner)))
            }),

            Shape3dField::Twist {
                shape_id,
                axis,
                center,
                ..
            } => of(shape_id).map(|child| {
                let plane = plane_axes(*axis);
                let reach = child
                    .corners()
                    .map(|corner| {
                        let offset = corner - *center;
                        TyVector2F64::new(offset[plane[0]], offset[plane[1]]).length()
                    })
                    .into_iter()
                    .fold(0.0, f64::max);

                let mut bounds = Bounds3d::around(*center, TyVector3F64::splat(reach));
                bounds.min[axis.index()] = child.min[axis.index()];
                bounds.max[axis.index()] = child.max[axis.index()];
                bounds
            }),

            Shape3dField::Union { shape_ids } => shape_ids
                .iter()
                .map(of)
                .reduce(|bounds, shape| {
                    bounds
                        .zip(shape)
                        .map(|(bounds, shape)| bounds.union(&shape))
                })
                .expect("a union holds a shape"),
        }
    }
}

fn primitive(distance: f64, point: TyVector3F64) -> SdfEvaluation {
    SdfEvaluation {
        distance,
        frame_position: point,
    }
}

fn nearer(nearest: SdfEvaluation, evaluation: SdfEvaluation) -> SdfEvaluation {
    if evaluation.distance < nearest.distance {
        evaluation
    } else {
        nearest
    }
}

fn unit(vector: TyVector3F64) -> TyVector3F64 {
    vector / vector.length()
}

fn revolve_point(
    point: TyVector3F64,
    axis: usize,
    plane: [usize; 2],
    center: TyVector3F64,
) -> TyVector2F64 {
    let offset = point - center;

    TyVector2F64::new(
        TyVector2F64::new(offset[plane[0]], offset[plane[1]]).length(),
        offset[axis],
    )
}

fn revolve_bounds(profile: &Bounds2d, axis: usize, center: TyVector3F64) -> Bounds3d {
    let mut bounds = Bounds3d::around(center, TyVector3F64::splat(profile.max.x.max(0.0)));
    bounds.min[axis] = center[axis] + profile.min.y;
    bounds.max[axis] = center[axis] + profile.max.y;
    bounds
}

/// The box around the disk of `radius` about `center` across the unit `axis`.
fn disk_bounds(center: TyVector3F64, axis: TyVector3F64, radius: f64) -> Bounds3d {
    let reach = (TyVector3F64::ONE - axis * axis).max(TyVector3F64::ZERO);
    let half_extents = TyVector3F64::new(reach.x.sqrt(), reach.y.sqrt(), reach.z.sqrt()) * radius;

    Bounds3d::around(center, half_extents)
}

/// The maps of a `mirror` in binary order with x as the low bit. The order
/// decides which copy wins a tie.
fn mirror_maps(axes: SdfAxes3d, center: TyVector3F64) -> Vec<PointMap3d> {
    let flags: u32 = match axes {
        SdfAxes3d::X => 0b001,
        SdfAxes3d::Xy => 0b011,
        SdfAxes3d::Xyz => 0b111,
        SdfAxes3d::Xz => 0b101,
        SdfAxes3d::Y => 0b010,
        SdfAxes3d::Yz => 0b110,
        SdfAxes3d::Z => 0b100,
    };

    (0..8_u32)
        .filter(|mask| mask & !flags == 0)
        .map(|mask| PointMap3d::Reflection {
            axes: [mask & 0b001 != 0, mask & 0b010 != 0, mask & 0b100 != 0],
            center,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::{
        SweepSample, assert_shape3d, bounds_of, box_reference, fbm, grid3d, rect_reference,
        segment_distance3d, shapes_of,
    };
    use branded_id::U32Id;
    use sdfcore::{BSdfShape2d, BSdfShape3d, SdfAxes3d, SdfShape2d, SdfShape3d, SdfSide};
    use std::f64::consts::FRAC_PI_2;
    use ty_math::{TyAxis3, TyVector2F64, TyVector3F64};

    const GRID: (TyVector3F64, TyVector3F64, u32) =
        (TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9);

    fn id(index: u32) -> U32Id<BSdfShape3d> {
        U32Id::from_u32(index)
    }

    fn profile_id(index: u32) -> U32Id<BSdfShape2d> {
        U32Id::from_u32(index)
    }

    fn sphere(x: f64, y: f64, z: f64, radius: f64) -> SdfShape3d {
        SdfShape3d::Sphere {
            center: TyVector3F64::new(x, y, z),
            radius,
        }
    }

    fn cuboid(min: TyVector3F64, max: TyVector3F64) -> SdfShape3d {
        SdfShape3d::Box {
            min,
            max,
            round: None,
        }
    }

    /// The distance from `point` to the torus of `ring_radius` and
    /// `tube_radius` about the unit `axis` through `center`, through the
    /// nearest point of its ring.
    fn torus_reference(
        point: TyVector3F64,
        center: TyVector3F64,
        axis: TyVector3F64,
        ring_radius: f64,
        tube_radius: f64,
    ) -> f64 {
        let offset = point - center;
        let across = offset - axis * offset.dot(axis);
        let ring = center + across.normalize_or(axis.any_orthonormal_vector()) * ring_radius;

        point.distance(ring) - tube_radius
    }

    #[test]
    fn a_sphere_and_a_capsule_measure_exact_distances() {
        assert_shape3d(
            Vec::new(),
            vec![sphere(0.1, -0.2, 0.05, 0.6)],
            GRID,
            1e-15,
            |point| point.distance(TyVector3F64::new(0.1, -0.2, 0.05)) - 0.6,
        );

        let a = TyVector3F64::new(0.1, -0.4, 0.2);
        let b = TyVector3F64::new(-0.3, 0.5, 0.1);
        let capsule = SdfShape3d::Capsule { a, b, radius: 0.3 };

        assert_shape3d(Vec::new(), vec![capsule], GRID, 1e-12, |point| {
            segment_distance3d(point, a, b) - 0.3
        });
    }

    #[test]
    fn a_cylinder_measures_exact_distances_at_any_angle() {
        let a = TyVector3F64::new(0.1, -0.4, 0.2);
        let b = TyVector3F64::new(-0.3, 0.5, 0.1);
        let axis = (b - a).normalize();
        let half_length = a.distance(b) / 2.0;

        for (radius, round) in [(0.4, None), (0.4, Some(0.1))] {
            let shape = SdfShape3d::Cylinder {
                a,
                b,
                radius,
                round,
            };
            let round = round.unwrap_or(0.0);

            assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
                let offset = point - (a + b) / 2.0;
                let profile = TyVector2F64::new(offset.cross(axis).length(), offset.dot(axis));
                let half_extents = TyVector2F64::new(radius, half_length) - round;

                rect_reference(profile, -half_extents, half_extents) - round
            });
        }
    }

    #[test]
    fn a_torus_rings_its_axis() {
        let center = TyVector3F64::new(0.1, -0.2, 0.05);

        for (axis, unit) in [
            (None, TyVector3F64::Y),
            (Some(TyAxis3::X), TyVector3F64::X),
            (Some(TyAxis3::Z), TyVector3F64::Z),
        ] {
            let shape = SdfShape3d::Torus {
                center,
                ring_radius: 0.5,
                tube_radius: 0.2,
                axis,
                from: None,
                to: None,
            };

            assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
                torus_reference(point, center, unit, 0.5, 0.2)
            });
        }
    }

    #[test]
    fn a_cut_torus_ends_round_at_its_angles() {
        let center = TyVector3F64::new(0.1, -0.2, 0.05);
        let (from, to) = (30.0, 250.0);
        let ring_point = |degrees: f64| {
            let (sin, cos) = f64::to_radians(degrees).sin_cos();
            center + TyVector3F64::new(cos, sin, 0.0) * 0.5
        };

        // About z, the angles run from +x toward +y.
        let shape = SdfShape3d::Torus {
            center,
            ring_radius: 0.5,
            tube_radius: 0.2,
            axis: Some(TyAxis3::Z),
            from: Some(from),
            to: Some(to),
        };

        assert_shape3d(Vec::new(), vec![shape], GRID, 1e-12, |point| {
            let offset = point - center;
            let angle = offset.y.atan2(offset.x).to_degrees();

            if (angle - from).rem_euclid(360.0) <= to - from {
                torus_reference(point, center, TyVector3F64::Z, 0.5, 0.2)
            } else {
                point
                    .distance(ring_point(from))
                    .min(point.distance(ring_point(to)))
                    - 0.2
            }
        });
    }

    #[test]
    fn a_half_space_holds_everything_past_its_plane_and_has_no_box() {
        let above = SdfShape3d::HalfSpace {
            side: SdfSide::PositiveY,
            at: 0.25,
        };
        let below = SdfShape3d::HalfSpace {
            side: SdfSide::NegativeX,
            at: -0.5,
        };

        assert_shape3d(Vec::new(), vec![above], GRID, 0.0, |point| 0.25 - point.y);
        assert_shape3d(Vec::new(), vec![below], GRID, 0.0, |point| point.x + 0.5);

        let (_, bounds) = bounds_of(
            &[],
            &[
                SdfShape3d::HalfSpace {
                    side: SdfSide::PositiveY,
                    at: 0.25,
                },
                sphere(0.0, 0.0, 0.0, 0.5),
                SdfShape3d::Intersect {
                    shape_ids: vec![id(0), id(1)],
                },
            ],
        );

        assert_eq!(bounds[id(0).to_usize_id()], None);
        assert_eq!(bounds[id(2).to_usize_id()], bounds[id(1).to_usize_id()]);
    }

    #[test]
    fn an_extrude_pushes_its_profile_along_the_axis() {
        let circle = SdfShape2d::Circle {
            center: TyVector2F64::new(0.1, -0.2),
            radius: 0.4,
        };

        assert_shape3d(
            vec![circle],
            vec![SdfShape3d::Extrude {
                profile_id: profile_id(0),
                axis: None,
                from: -0.3,
                to: 0.5,
            }],
            GRID,
            1e-12,
            |point| {
                let radial = TyVector2F64::new(point.x - 0.1, point.y + 0.2).length();
                rect_reference(
                    TyVector2F64::new(radial, point.z),
                    TyVector2F64::new(-0.4, -0.3),
                    TyVector2F64::new(0.4, 0.5),
                )
            },
        );

        // Across y, the profile's u and v map to x and z.
        let rect = SdfShape2d::Rect {
            min: TyVector2F64::new(-0.5, -0.25),
            max: TyVector2F64::new(0.25, 0.5),
            chamfer: None,
            round: None,
        };

        assert_shape3d(
            vec![rect],
            vec![SdfShape3d::Extrude {
                profile_id: profile_id(0),
                axis: Some(TyAxis3::Y),
                from: -0.3,
                to: 0.5,
            }],
            GRID,
            1e-12,
            |point| {
                box_reference(
                    point,
                    TyVector3F64::new(-0.5, -0.3, -0.25),
                    TyVector3F64::new(0.25, 0.5, 0.5),
                )
            },
        );
    }

    #[test]
    fn revolve_and_lathe_spin_their_profile_about_the_axis() {
        let circle = SdfShape2d::Circle {
            center: TyVector2F64::new(0.5, 0.1),
            radius: 0.2,
        };
        let center = TyVector3F64::new(0.1, -0.2, 0.05);

        assert_shape3d(
            vec![circle],
            vec![SdfShape3d::Revolve {
                profile_id: profile_id(0),
                axis: None,
                center: Some(center),
            }],
            GRID,
            1e-12,
            |point| {
                torus_reference(
                    point,
                    center + TyVector3F64::new(0.0, 0.1, 0.0),
                    TyVector3F64::Y,
                    0.5,
                    0.2,
                )
            },
        );

        let lathe = SdfShape3d::Lathe {
            points: vec![TyVector2F64::new(0.5, -0.25), TyVector2F64::new(0.5, 0.75)],
            axis: Some(TyAxis3::X),
            center: None,
        };

        assert_shape3d(Vec::new(), vec![lathe.clone()], GRID, 1e-12, |point| {
            rect_reference(
                TyVector2F64::new(TyVector2F64::new(point.y, point.z).length(), point.x),
                TyVector2F64::new(-0.5, -0.25),
                TyVector2F64::new(0.5, 0.75),
            )
        });

        // A shell of a lathe stays hollow along the axis.
        let shapes = shapes_of(
            Vec::new(),
            vec![
                lathe,
                SdfShape3d::Shell {
                    shape_id: id(0),
                    thickness: 0.1,
                },
            ],
        );
        assert!(
            shapes
                .evaluate(id(1), TyVector3F64::new(0.25, 0.0, 0.0))
                .distance
                > 0.0
        );
    }

    #[test]
    fn a_union_takes_the_frame_of_the_operand_it_sits_deepest_in() {
        let shapes = shapes_of(
            Vec::new(),
            vec![
                sphere(0.0, 0.0, 0.0, 0.5),
                SdfShape3d::Translate {
                    shape_id: id(0),
                    offset: TyVector3F64::new(1.0, 0.0, 0.0),
                },
                SdfShape3d::Union {
                    shape_ids: vec![id(0), id(1)],
                },
                SdfShape3d::SmoothUnion {
                    radius: 0.2,
                    shape_ids: vec![id(0), id(1)],
                },
                SdfShape3d::Intersect {
                    shape_ids: vec![id(1), id(0)],
                },
                SdfShape3d::Subtract {
                    base_id: id(1),
                    cutter_ids: vec![id(0)],
                },
            ],
        );

        let near_first = TyVector3F64::new(0.2, 0.1, 0.0);
        let near_second = TyVector3F64::new(0.9, 0.1, 0.0);
        let moved_back = near_second - TyVector3F64::X;

        for union_id in [id(2), id(3)] {
            assert_eq!(
                shapes.evaluate(union_id, near_first).frame_position,
                near_first
            );
            assert_eq!(
                shapes.evaluate(union_id, near_second).frame_position,
                moved_back
            );
        }

        // The middle point ties, and the earlier operand wins.
        let middle = TyVector3F64::new(0.5, 0.0, 0.0);
        assert_eq!(shapes.evaluate(id(2), middle).frame_position, middle);

        for first_id in [id(4), id(5)] {
            assert_eq!(
                shapes.evaluate(first_id, near_first).frame_position,
                near_first - TyVector3F64::X
            );
        }
    }

    #[test]
    fn a_quarter_turn_moves_a_box_exactly() {
        let shapes = shapes_of(
            Vec::new(),
            vec![
                cuboid(
                    TyVector3F64::new(0.0, 0.0, 0.0),
                    TyVector3F64::new(0.5, 0.25, 0.75),
                ),
                SdfShape3d::Rotate {
                    shape_id: id(0),
                    axis: TyAxis3::Y,
                    degrees: 90.0,
                    pivot: None,
                },
                cuboid(
                    TyVector3F64::new(0.0, 0.0, -0.5),
                    TyVector3F64::new(0.75, 0.25, 0.0),
                ),
            ],
        );

        for point in grid3d(TyVector3F64::splat(-1.0), TyVector3F64::splat(1.0), 9) {
            assert_eq!(
                shapes.evaluate(id(1), point).distance,
                shapes.evaluate(id(2), point).distance
            );
        }
    }

    #[test]
    fn transforms_move_turn_and_scale_about_their_pivot() {
        let pivot = TyVector3F64::new(0.1, 0.1, -0.1);

        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.2, 0.0, 0.0, 0.3),
                SdfShape3d::Translate {
                    shape_id: id(0),
                    offset: TyVector3F64::new(-0.3, 0.2, 0.1),
                },
            ],
            GRID,
            1e-15,
            |point| point.distance(TyVector3F64::new(-0.1, 0.2, 0.1)) - 0.3,
        );

        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.1, 0.5, -0.1, 0.3),
                SdfShape3d::Rotate {
                    shape_id: id(0),
                    axis: TyAxis3::X,
                    degrees: 30.0,
                    pivot: Some(pivot),
                },
            ],
            GRID,
            1e-15,
            |point| {
                let (sin, cos) = 30.0_f64.to_radians().sin_cos();
                point.distance(pivot + TyVector3F64::new(0.0, 0.4 * cos, 0.4 * sin)) - 0.3
            },
        );

        assert_shape3d(
            Vec::new(),
            vec![
                SdfShape3d::Cylinder {
                    a: TyVector3F64::new(0.0, -0.5, 0.0),
                    b: TyVector3F64::new(0.0, 0.5, 0.0),
                    radius: 0.25,
                    round: None,
                },
                SdfShape3d::Orient {
                    shape_id: id(0),
                    from: TyVector3F64::new(0.0, 2.0, 0.0),
                    to: TyVector3F64::new(3.0, 0.0, 0.0),
                    pivot: None,
                },
            ],
            GRID,
            1e-15,
            |point| {
                rect_reference(
                    TyVector2F64::new(TyVector2F64::new(point.y, point.z).length(), point.x),
                    TyVector2F64::new(-0.25, -0.5),
                    TyVector2F64::new(0.25, 0.5),
                )
            },
        );

        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.2, 0.0, 0.0, 0.3),
                SdfShape3d::Scale {
                    shape_id: id(0),
                    factor: TyVector3F64::splat(2.0),
                    pivot: Some(pivot),
                },
            ],
            GRID,
            1e-15,
            |point| point.distance(TyVector3F64::new(0.3, -0.1, 0.1)) - 0.6,
        );

        // A non-uniform scale takes its least factor, which reads short.
        let factor = TyVector3F64::new(2.0, 0.5, 1.0);
        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.2, 0.0, 0.0, 0.3),
                SdfShape3d::Scale {
                    shape_id: id(0),
                    factor,
                    pivot: None,
                },
            ],
            GRID,
            1e-15,
            |point| 0.5 * ((point / factor).distance(TyVector3F64::new(0.2, 0.0, 0.0)) - 0.3),
        );
    }

    #[test]
    fn copies_union_every_place() {
        let nearest = |centers: &[TyVector3F64], point: TyVector3F64| {
            centers
                .iter()
                .map(|center| point.distance(*center) - 0.15)
                .fold(f64::INFINITY, f64::min)
        };

        let mirrored = [
            (0.3, 0.2, 0.1),
            (-0.3, 0.2, 0.1),
            (0.3, 0.2, -0.1),
            (-0.3, 0.2, -0.1),
        ]
        .map(|(x, y, z)| TyVector3F64::new(x, y, z));
        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.3, 0.2, 0.1, 0.15),
                SdfShape3d::Mirror {
                    shape_id: id(0),
                    axes: SdfAxes3d::Xz,
                    center: None,
                },
            ],
            GRID,
            1e-15,
            |point| nearest(&mirrored, point),
        );

        let repeated = [(0.3, 0.2, 0.1), (0.3, 0.2, -0.2), (0.3, 0.2, -0.5)]
            .map(|(x, y, z)| TyVector3F64::new(x, y, z));
        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.3, 0.2, 0.1, 0.15),
                SdfShape3d::Repeat {
                    shape_id: id(0),
                    step: TyVector3F64::new(0.5, 0.5, -0.3),
                    count: TyVector3F64::new(1.0, 1.0, 3.0),
                },
            ],
            GRID,
            1e-15,
            |point| nearest(&repeated, point),
        );

        // Four turns about y take +x to -z, -x, and +z.
        let turned = [
            (0.3, 0.2, 0.1),
            (0.1, 0.2, -0.3),
            (-0.3, 0.2, -0.1),
            (-0.1, 0.2, 0.3),
        ]
        .map(|(x, y, z)| TyVector3F64::new(x, y, z));
        assert_shape3d(
            Vec::new(),
            vec![
                sphere(0.3, 0.2, 0.1, 0.15),
                SdfShape3d::RepeatPolar {
                    shape_id: id(0),
                    axis: TyAxis3::Y,
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
    fn modifiers_move_the_surface() {
        let ball = |point: TyVector3F64| point.length() - 0.5;
        let with = |shape: SdfShape3d| vec![sphere(0.0, 0.0, 0.0, 0.5), shape];

        assert_shape3d(
            Vec::new(),
            with(SdfShape3d::Offset {
                shape_id: id(0),
                distance: 0.2,
            }),
            GRID,
            1e-15,
            |point| ball(point) - 0.2,
        );
        assert_shape3d(
            Vec::new(),
            with(SdfShape3d::Shell {
                shape_id: id(0),
                thickness: 0.1,
            }),
            GRID,
            1e-15,
            |point| ball(point).max(-ball(point) - 0.1),
        );

        // A sphere stretched along x makes a capsule.
        assert_shape3d(
            Vec::new(),
            with(SdfShape3d::Elongate {
                shape_id: id(0),
                lengths: TyVector3F64::new(0.6, 0.0, 0.0),
                center: None,
            }),
            GRID,
            1e-15,
            |point| {
                segment_distance3d(
                    point,
                    TyVector3F64::new(-0.3, 0.0, 0.0),
                    TyVector3F64::new(0.3, 0.0, 0.0),
                ) - 0.5
            },
        );

        assert_shape3d(
            Vec::new(),
            with(SdfShape3d::Displace {
                shape_id: id(0),
                amplitude: 0.05,
                scale: 0.2,
                octaves: None,
                seed: -3.0,
            }),
            GRID,
            1e-15,
            |point| ball(point) + 0.05 * fbm(point / 0.2, 4, -3_i32 as u32),
        );
    }

    #[test]
    fn a_twist_turns_each_slice_by_its_height() {
        let min = TyVector3F64::new(-0.4, -0.75, -0.2);
        let max = TyVector3F64::new(0.4, 0.75, 0.2);
        let center = TyVector3F64::new(0.05, 0.1, 0.0);

        let shapes = vec![
            cuboid(min, max),
            SdfShape3d::Twist {
                shape_id: id(0),
                axis: TyAxis3::Y,
                degrees_per_meter: 60.0,
                center: Some(center),
            },
        ];

        let reference = |point: TyVector3F64| {
            // The slice at the point's height turned back about y.
            let (sin, cos) = (-60.0 * (point.y - center.y)).to_radians().sin_cos();
            let offset = point - center;
            let turned = TyVector3F64::new(
                offset.x * cos + offset.z * sin,
                offset.y,
                offset.z * cos - offset.x * sin,
            );
            box_reference(turned + center, min, max)
        };

        assert_shape3d(Vec::new(), shapes.clone(), GRID, 1e-12, reference);

        let twisted = shapes_of(Vec::new(), shapes);
        let point = TyVector3F64::new(0.3, 0.6, 0.1);
        let frame_position = twisted.evaluate(id(1), point).frame_position;
        assert!((box_reference(frame_position, min, max) - reference(point)).abs() < 1e-12);
    }

    #[test]
    fn a_bend_curls_its_axis_into_an_arc_that_keeps_lengths() {
        // A bar a quarter turn long, bent toward +y about the circle of radius
        // 1 through [0, 1, 0].
        let length = FRAC_PI_2;
        let min = TyVector3F64::new(0.0, -0.1, -0.2);
        let max = TyVector3F64::new(length, 0.1, 0.2);

        let shapes = shapes_of(
            Vec::new(),
            vec![
                cuboid(min, max),
                SdfShape3d::Bend {
                    shape_id: id(0),
                    along: TyAxis3::X,
                    toward: SdfSide::PositiveY,
                    radius: 1.0,
                    pivot: None,
                },
            ],
        );

        for step in 0..=10 {
            let angle = length * f64::from(step) / 10.0 - 0.2;

            for radius in [0.8, 0.95, 1.0, 1.05, 1.3] {
                for z in [-0.3, 0.0, 0.15] {
                    let point =
                        TyVector3F64::new(radius * angle.sin(), 1.0 - radius * angle.cos(), z);
                    let unbent = TyVector3F64::new(angle, 1.0 - radius, z);
                    let distance = shapes.evaluate(id(1), point).distance;

                    assert!(
                        (distance - box_reference(unbent, min, max)).abs() < 1e-12,
                        "{point}"
                    );
                }
            }
        }
    }

    #[test]
    fn a_straight_sweep_turns_its_profile_s_u_right_and_v_up() {
        let rect = SdfShape2d::Rect {
            min: TyVector2F64::new(-0.5, -0.25),
            max: TyVector2F64::new(0.25, 0.5),
            chamfer: None,
            round: None,
        };
        let sweep = |a: TyVector3F64, b: TyVector3F64, up: Option<TyVector3F64>| {
            vec![SdfShape3d::Sweep {
                profile_id: profile_id(0),
                points: vec![a, b],
                scales: None,
                up,
            }]
        };
        let x = TyVector3F64::new(0.6, 0.0, 0.0);
        let z = TyVector3F64::new(0.0, 0.0, 0.6);

        for (shapes3d, min, max) in [
            (
                sweep(-x, x, None),
                TyVector3F64::new(-0.6, -0.25, -0.5),
                TyVector3F64::new(0.6, 0.5, 0.25),
            ),
            (
                sweep(-z, z, None),
                TyVector3F64::new(-0.25, -0.25, -0.6),
                TyVector3F64::new(0.5, 0.5, 0.6),
            ),
            (
                sweep(-x, x, Some(TyVector3F64::new(0.0, 0.0, 2.0))),
                TyVector3F64::new(-0.6, -0.25, -0.25),
                TyVector3F64::new(0.6, 0.5, 0.5),
            ),
        ] {
            let shapes = shapes_of(vec![rect.clone()], shapes3d);

            for point in grid3d(GRID.0, GRID.1, GRID.2) {
                let distance = shapes.evaluate(id(0), point).distance;
                let expected = box_reference(point, min, max);

                // Inside, the caps leave the depth to the profile.
                if expected > 0.0 {
                    assert!((distance - expected).abs() < 1e-12, "{point}");
                } else {
                    assert!(distance <= 1e-12, "{point}");
                }
            }
        }
    }

    #[test]
    fn a_sweep_s_tube_keeps_its_radius_around_a_bend_and_tapers() {
        let points = vec![
            TyVector3F64::new(-0.6, -0.2, 0.0),
            TyVector3F64::new(0.0, 0.3, 0.1),
            TyVector3F64::new(0.6, -0.2, 0.0),
        ];
        let circle = SdfShape2d::Circle {
            center: TyVector2F64::ZERO,
            radius: 0.2,
        };
        let samples = SweepSample::along(&points, None);
        let shapes = shapes_of(
            vec![circle.clone()],
            vec![SdfShape3d::Sweep {
                profile_id: profile_id(0),
                points: points.clone(),
                scales: None,
                up: None,
            }],
        );

        for point in grid3d(GRID.0, GRID.1, GRID.2) {
            let curve = samples
                .windows(2)
                .map(|pair| segment_distance3d(point, pair[0].position, pair[1].position))
                .fold(f64::INFINITY, f64::min);

            // The tube reference leaves out the flat caps at the ends.
            if [points[0], points[2]]
                .iter()
                .all(|end| point.distance(*end) > curve + 1e-9)
            {
                let distance = shapes.evaluate(id(0), point).distance;
                assert!((distance - (curve - 0.2)).abs() < 5e-3, "{point}");
            }
        }

        // Scales halve the radius halfway along a straight sweep.
        let shapes = shapes_of(
            vec![circle],
            vec![SdfShape3d::Sweep {
                profile_id: profile_id(0),
                points: vec![
                    TyVector3F64::new(-0.6, 0.0, 0.0),
                    TyVector3F64::new(0.6, 0.0, 0.0),
                ],
                scales: Some(vec![1.0, 0.0]),
                up: None,
            }],
        );

        assert!(
            shapes
                .evaluate(id(0), TyVector3F64::new(0.0, 0.09, 0.0))
                .distance
                < 0.0
        );
        assert!(
            shapes
                .evaluate(id(0), TyVector3F64::new(0.0, 0.11, 0.0))
                .distance
                > 0.0
        );
        assert!(
            shapes
                .evaluate(id(0), TyVector3F64::new(0.59, 0.0, 0.0))
                .distance
                < 0.0
        );
        assert!(
            shapes
                .evaluate(id(0), TyVector3F64::new(0.61, 0.0, 0.0))
                .distance
                > 0.0
        );
    }
}
