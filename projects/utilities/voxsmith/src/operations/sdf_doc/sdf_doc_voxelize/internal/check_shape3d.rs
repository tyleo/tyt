use crate::operations::sdf_doc::{ArgumentCheck, SweepSample, side_direction};
use branded_id::IdVec;
use sdfcore::{BSdfShape3d, SdfShape3d, SdfSide};
use std::result::Result as StdResult;
use ty_math::TyVector3F64;

/// Checks the arguments of the 3D shape `shape`. `has_box` reads whether each
/// shape has a box.
pub fn check_shape3d(
    shape: &SdfShape3d,
    has_box: &IdVec<BSdfShape3d, bool>,
) -> StdResult<(), String> {
    match shape {
        SdfShape3d::Bend {
            shape_id,
            along,
            toward,
            radius,
            ..
        } => {
            let check = ArgumentCheck::new("bend");
            check.above_zero("radius", *radius)?;
            check.expect(
                side_direction(*toward).0 != *along,
                "toward",
                &format!("on another axis than along {along}"),
                side_name(*toward),
            )?;
            check.expect(
                has_box[shape_id.to_usize_id()],
                "shape",
                "bounded",
                "a shape that reaches without end",
            )
        }

        SdfShape3d::Box { min, max, round } => {
            let check = ArgumentCheck::new("box");
            check.opposite_corners(*min, *max)?;

            match round {
                Some(round) => {
                    let most = (*max - *min).abs().min_element() / 2.0;
                    check.at_least_zero("round", *round)?;
                    check.at_most(
                        "round",
                        *round,
                        most,
                        &format!("at most half the shortest side, {most}"),
                    )
                }

                None => Ok(()),
            }
        }

        SdfShape3d::BoxFrame {
            min,
            max,
            thickness,
        } => {
            let check = ArgumentCheck::new("boxFrame");
            check.opposite_corners(*min, *max)?;
            check.above_zero("thickness", *thickness)
        }

        SdfShape3d::Capsule { a, b, radius } => {
            let check = ArgumentCheck::new("capsule");
            check.above_zero("radius", *radius)?;
            ends(check, *a, *b)
        }

        SdfShape3d::Cone {
            a,
            b,
            radius_a,
            radius_b,
        } => {
            let check = ArgumentCheck::new("cone");
            check.at_least_zero("radiusA", *radius_a)?;
            check.at_least_zero("radiusB", *radius_b)?;
            ends(check, *a, *b)
        }

        SdfShape3d::Cylinder {
            a,
            b,
            radius,
            round,
        } => {
            let check = ArgumentCheck::new("cylinder");
            check.above_zero("radius", *radius)?;
            ends(check, *a, *b)?;

            match round {
                Some(round) => {
                    let most = radius.min(a.distance(*b) / 2.0);
                    check.at_least_zero("round", *round)?;
                    check.at_most(
                        "round",
                        *round,
                        most,
                        &format!("at most the radius and half the length, {most}"),
                    )
                }

                None => Ok(()),
            }
        }

        SdfShape3d::Displace {
            amplitude,
            scale,
            octaves,
            seed,
            ..
        } => {
            let check = ArgumentCheck::new("displace");
            check.at_least_zero("amplitude", *amplitude)?;
            check.above_zero("scale", *scale)?;

            if let Some(octaves) = octaves {
                check.whole_above_zero("octaves", *octaves)?;
            }

            check.seed(*seed)
        }

        SdfShape3d::Elongate { lengths, .. } => ArgumentCheck::new("elongate").expect(
            lengths.cmpge(TyVector3F64::ZERO).all(),
            "lengths",
            "zero or more on each axis",
            format!("[{}, {}, {}]", lengths.x, lengths.y, lengths.z),
        ),

        SdfShape3d::Ellipsoid { radii, .. } => {
            ArgumentCheck::new("ellipsoid").each_above_zero("radii", *radii)
        }

        SdfShape3d::Extrude { from, to, .. } => ArgumentCheck::new("extrude").range(*from, *to),

        SdfShape3d::HalfSpace { .. }
        | SdfShape3d::Mirror { .. }
        | SdfShape3d::Offset { .. }
        | SdfShape3d::Revolve { .. }
        | SdfShape3d::Rotate { .. }
        | SdfShape3d::Subtract { .. }
        | SdfShape3d::Translate { .. }
        | SdfShape3d::Twist { .. } => Ok(()),

        SdfShape3d::Intersect { shape_ids } => {
            ArgumentCheck::new("intersect").count("shapes", shape_ids.len(), 1)
        }

        SdfShape3d::Lathe { points, .. } => {
            ArgumentCheck::new("lathe").count("points", points.len(), 2)
        }

        SdfShape3d::Octahedron { radius, .. } => {
            ArgumentCheck::new("octahedron").above_zero("radius", *radius)
        }

        SdfShape3d::Orient { from, to, .. } => {
            let check = ArgumentCheck::new("orient");
            check.expect(from.length() > 0.0, "from", "a direction", "a zero vector")?;
            check.expect(to.length() > 0.0, "to", "a direction", "a zero vector")?;
            check.expect(
                1.0 + from.normalize().dot(to.normalize()) > 0.0,
                "to",
                "a direction other than the opposite of from",
                format!("[{}, {}, {}]", to.x, to.y, to.z),
            )
        }

        SdfShape3d::Pyramid { width, height, .. } => {
            let check = ArgumentCheck::new("pyramid");
            check.above_zero("width", *width)?;
            check.above_zero("height", *height)
        }

        SdfShape3d::Repeat { count, .. } => {
            let check = ArgumentCheck::new("repeat");

            for value in count.to_array() {
                check.whole_above_zero("count", value)?;
            }

            Ok(())
        }

        SdfShape3d::RepeatPolar { count, .. } => {
            ArgumentCheck::new("repeatPolar").whole_above_zero("count", *count)
        }

        SdfShape3d::RoundCone {
            a,
            b,
            radius_a,
            radius_b,
        } => {
            let check = ArgumentCheck::new("roundCone");
            check.above_zero("radiusA", *radius_a)?;
            check.above_zero("radiusB", *radius_b)?;
            ends(check, *a, *b)?;

            let length = a.distance(*b);
            check.expect(
                (radius_a - radius_b).abs() < length,
                "radiusB",
                &format!("within {length} of radiusA {radius_a}"),
                radius_b,
            )
        }

        SdfShape3d::Scale { factor, .. } => {
            ArgumentCheck::new("scale").each_nonzero("factor", *factor)
        }

        SdfShape3d::Shell { thickness, .. } => {
            ArgumentCheck::new("shell").above_zero("thickness", *thickness)
        }

        SdfShape3d::SmoothIntersect { radius, shape_ids } => {
            let check = ArgumentCheck::new("smoothIntersect");
            check.above_zero("radius", *radius)?;
            check.count("shapes", shape_ids.len(), 1)
        }

        SdfShape3d::SmoothSubtract { radius, .. } => {
            ArgumentCheck::new("smoothSubtract").above_zero("radius", *radius)
        }

        SdfShape3d::SmoothUnion { radius, shape_ids } => {
            let check = ArgumentCheck::new("smoothUnion");
            check.above_zero("radius", *radius)?;
            check.count("shapes", shape_ids.len(), 1)
        }

        SdfShape3d::Sphere { radius, .. } => {
            ArgumentCheck::new("sphere").above_zero("radius", *radius)
        }

        SdfShape3d::Sweep {
            points, scales, up, ..
        } => {
            let check = ArgumentCheck::new("sweep");
            check.count("points", points.len(), 2)?;

            if let Some(pair) = points.windows(2).find(|pair| pair[0] == pair[1]) {
                return Err(check.fail(
                    "points",
                    "distinct from their neighbors",
                    format!("{} twice in a row", vector(pair[0])),
                ));
            }

            if let Some(scales) = scales {
                check.expect(
                    scales.len() == points.len(),
                    "scales",
                    &format!("a list of {}, one per point", points.len()),
                    format!("a list of {}", scales.len()),
                )?;

                for scale in scales {
                    check.at_least_zero("scales", *scale)?;
                }
            }

            let up = up.unwrap_or(TyVector3F64::Y);
            check.expect(up.length() > 0.0, "up", "a direction", "a zero vector")?;

            let samples = SweepSample::along(points, scales.as_deref());
            check.expect(
                samples.iter().all(|sample| sample.scale >= 0.0),
                "scales",
                "scales the curve keeps at 0 or more",
                "scales the curve takes below 0",
            )?;
            check.expect(
                samples
                    .windows(2)
                    .all(|pair| pair[0].tangent.cross(up).dot(pair[1].tangent.cross(up)) > 0.0),
                "up",
                "a direction the curve never runs along",
                vector(up),
            )
        }

        SdfShape3d::Torus {
            ring_radius,
            tube_radius,
            from,
            to,
            ..
        } => {
            let check = ArgumentCheck::new("torus");
            check.above_zero("ringRadius", *ring_radius)?;
            check.above_zero("tubeRadius", *tube_radius)?;

            match (from, to) {
                (Some(from), Some(to)) => {
                    check.range(*from, *to)?;
                    check.expect(
                        to - from <= 360.0,
                        "to",
                        &format!("at most 360 past from {from}"),
                        to,
                    )
                }

                (None, None) => Ok(()),

                (Some(from), None) => Err(check.fail(
                    "to",
                    "given with from",
                    format!("left out beside from {from}"),
                )),

                (None, Some(to)) => {
                    Err(check.fail("from", "given with to", format!("left out beside to {to}")))
                }
            }
        }

        SdfShape3d::Union { shape_ids } => {
            ArgumentCheck::new("union").count("shapes", shape_ids.len(), 1)
        }
    }
}

/// Errors unless the ends `a` and `b` differ.
fn ends(check: ArgumentCheck, a: TyVector3F64, b: TyVector3F64) -> StdResult<(), String> {
    check.expect(a != b, "b", "another point than a", vector(b))
}

/// `value` as the TypeScript API writes it.
fn vector(value: TyVector3F64) -> String {
    format!("[{}, {}, {}]", value.x, value.y, value.z)
}

/// `side` as the TypeScript API writes it.
fn side_name(side: SdfSide) -> &'static str {
    match side {
        SdfSide::NegativeX => "-x",
        SdfSide::NegativeY => "-y",
        SdfSide::NegativeZ => "-z",
        SdfSide::PositiveX => "+x",
        SdfSide::PositiveY => "+y",
        SdfSide::PositiveZ => "+z",
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::sdf_doc::check_shape3d;
    use branded_id::{IdVec, U32Id};
    use sdfcore::{SdfShape3d, SdfSide};
    use ty_math::{TyAxis3, TyVector3F64};

    fn check(shape: SdfShape3d) -> Result<(), String> {
        check_shape3d(&shape, &IdVec::from_vec(vec![true, false]))
    }

    #[test]
    fn a_cut_torus_takes_both_angles_within_a_turn() {
        let torus = |from, to| SdfShape3d::Torus {
            center: TyVector3F64::ZERO,
            ring_radius: 1.0,
            tube_radius: 0.25,
            axis: None,
            from,
            to,
        };

        assert!(check(torus(Some(0.0), Some(360.0))).is_ok());
        assert_eq!(
            check(torus(Some(30.0), None)),
            Err("torus to must be given with from, not left out beside from 30".to_string())
        );
        assert_eq!(
            check(torus(Some(30.0), Some(400.0))),
            Err("torus to must be at most 360 past from 30, not 400".to_string())
        );
    }

    #[test]
    fn a_sweep_scales_each_point_and_never_runs_along_up() {
        let sweep = |points: &[[f64; 3]], scales: Option<Vec<f64>>, up| SdfShape3d::Sweep {
            profile_id: U32Id::from_u32(0),
            points: points.iter().copied().map(TyVector3F64::from).collect(),
            scales,
            up,
        };
        let line = [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];

        assert!(check(sweep(&line, Some(vec![1.0, 0.5, 0.2, 0.0]), None)).is_ok());
        assert_eq!(
            check(sweep(&line[..1], None, None)),
            Err("sweep points must be a list of at least 2, not a list of 1".to_string())
        );
        assert_eq!(
            check(sweep(&[[0.0, 0.0, 0.0], [0.0, 0.0, 0.0]], None, None)),
            Err(
                "sweep points must be distinct from their neighbors, not [0, 0, 0] twice in a row"
                    .to_string()
            )
        );
        assert_eq!(
            check(sweep(&line, Some(vec![1.0, 0.5]), None)),
            Err("sweep scales must be a list of 4, one per point, not a list of 2".to_string())
        );
        assert!(check(sweep(&line, Some(vec![1.0, -0.5, 0.2, 0.0]), None)).is_err());
        assert_eq!(
            check(sweep(&line, Some(vec![1.0, 1.0, 0.0, 0.0]), None)),
            Err(
                "sweep scales must be scales the curve keeps at 0 or more, not scales the curve \
                 takes below 0"
                    .to_string()
            )
        );

        // A curve that rises straight up, or turns through straight up, runs
        // along the default up.
        let rise = [[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let arch = [[-1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let turn = [[0.0, 0.0, 0.0], [0.5, 1.0, 0.0], [0.0, 2.0, 0.0]];
        assert_eq!(
            check(sweep(&rise, None, None)),
            Err(
                "sweep up must be a direction the curve never runs along, not [0, 1, 0]"
                    .to_string()
            )
        );
        assert!(check(sweep(&rise, None, Some(TyVector3F64::Z))).is_ok());
        assert!(check(sweep(&arch, None, None)).is_ok());
        assert!(check(sweep(&turn, None, None)).is_err());
        assert!(check(sweep(&arch, None, Some(TyVector3F64::ZERO))).is_err());
    }

    #[test]
    fn a_bend_turns_toward_another_axis_and_a_bounded_shape() {
        let bend = |shape, toward| SdfShape3d::Bend {
            shape_id: U32Id::from_u32(shape),
            along: TyAxis3::X,
            toward,
            radius: 1.0,
            pivot: None,
        };

        assert!(check(bend(0, SdfSide::PositiveY)).is_ok());
        assert_eq!(
            check(bend(0, SdfSide::NegativeX)),
            Err("bend toward must be on another axis than along x, not -x".to_string())
        );
        assert_eq!(
            check(bend(1, SdfSide::PositiveY)),
            Err("bend shape must be bounded, not a shape that reaches without end".to_string())
        );
    }

    #[test]
    fn orient_takes_two_directions_that_do_not_oppose() {
        let orient = |to| SdfShape3d::Orient {
            shape_id: U32Id::from_u32(0),
            from: TyVector3F64::Y,
            to,
            pivot: None,
        };

        assert!(check(orient(TyVector3F64::X)).is_ok());
        assert!(check(orient(TyVector3F64::ZERO)).is_err());
        assert_eq!(
            check(orient(TyVector3F64::new(0.0, -2.0, 0.0))),
            Err(
                "orient to must be a direction other than the opposite of from, not [0, -2, 0]"
                    .to_string()
            )
        );
    }

    #[test]
    fn a_box_takes_its_corners_in_either_order_but_apart_on_each_axis() {
        let cuboid = |min: [f64; 3], max: [f64; 3]| SdfShape3d::Box {
            min: TyVector3F64::from(min),
            max: TyVector3F64::from(max),
            round: Some(0.02),
        };

        assert!(check(cuboid([-0.2, 0.0, 0.0], [-0.25, 0.1, 0.1])).is_ok());
        assert_eq!(
            check(cuboid([0.0, 0.0, 0.0], [0.1, -0.1, 0.0])),
            Err(
                "box max must be apart from min [0, 0, 0] on each axis, not [0.1, -0.1, 0]"
                    .to_string()
            )
        );
        assert!(check(cuboid([0.0, 0.0, 0.0], [-0.03, 0.1, 0.1])).is_err());
    }

    #[test]
    fn a_box_round_may_pass_half_a_side_by_its_rounding() {
        let rounded = |round| SdfShape3d::Box {
            min: TyVector3F64::new(0.4, 0.0, 0.0),
            max: TyVector3F64::new(0.45, 0.1, 0.1),
            round: Some(round),
        };

        assert!(check(rounded(0.025)).is_ok());
        assert_eq!(
            check(rounded(0.026)),
            Err(
                "box round must be at most half the shortest side, 0.024999999999999994, not 0.026"
                    .to_string()
            )
        );
    }

    #[test]
    fn a_round_of_zero_leaves_the_edges_square() {
        let cuboid = |round| SdfShape3d::Box {
            min: TyVector3F64::ZERO,
            max: TyVector3F64::ONE,
            round: Some(round),
        };
        let cylinder = |round| SdfShape3d::Cylinder {
            a: TyVector3F64::ZERO,
            b: TyVector3F64::Y,
            radius: 0.5,
            round: Some(round),
        };

        assert!(check(cuboid(0.0)).is_ok());
        assert!(check(cylinder(0.0)).is_ok());
        assert_eq!(
            check(cuboid(-0.1)),
            Err("box round must be zero or more, not -0.1".to_string())
        );
        assert_eq!(
            check(cylinder(-0.1)),
            Err("cylinder round must be zero or more, not -0.1".to_string())
        );
    }

    #[test]
    fn an_elongate_of_zero_leaves_the_shape_unstretched() {
        let elongate = |lengths: [f64; 3]| SdfShape3d::Elongate {
            shape_id: U32Id::from_u32(0),
            lengths: TyVector3F64::from(lengths),
            center: None,
        };

        assert!(check(elongate([0.0, 0.0, 0.0])).is_ok());
        assert_eq!(
            check(elongate([-0.1, 0.0, 0.0])),
            Err("elongate lengths must be zero or more on each axis, not [-0.1, 0, 0]".to_string())
        );
    }

    #[test]
    fn a_round_cone_tapers_less_than_its_length() {
        let round_cone = |radius_b| SdfShape3d::RoundCone {
            a: TyVector3F64::ZERO,
            b: TyVector3F64::new(0.0, 1.0, 0.0),
            radius_a: 0.5,
            radius_b,
        };

        assert!(check(round_cone(1.25)).is_ok());
        assert_eq!(
            check(round_cone(1.5)),
            Err("roundCone radiusB must be within 1 of radiusA 0.5, not 1.5".to_string())
        );
    }
}
