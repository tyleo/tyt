use crate::{
    Error, Result,
    operations::object::{
        FitOrFixed, NodeFrames, PoseTransform, RenderElement, ViewProjection, look_rotation,
        pose_in_frame,
    },
};
use ty_math::{TyAngleUnit, TyBoundsF64, TyPoseF64, TyTransformF64, TyVector3Ext, TyVector3F64};
use voxrender::{RenderProjection, RenderView, fit_distance, fit_scale};

/// Resolves a view's `transform` and `projection` to world space. Errors if a
/// `subject` or `orbit` frame or a `fit` has no subject, a `node` frame's
/// glob matches no node path or several, or a look-at aims at the view's own
/// position.
///
/// # Arguments
/// * `element` - the view, which errors report.
/// * `subject` - the world bounds of the view's subject, or `None` when it
///   has no voxel.
/// * `width`, `height` - the image size in pixels, whose shorter axis a
///   `fit` fills.
pub fn resolve_view(
    element: &RenderElement,
    transform: &PoseTransform,
    projection: &ViewProjection,
    subject: Option<&TyBoundsF64>,
    node_frames: &NodeFrames,
    width: u32,
    height: u32,
) -> Result<RenderView> {
    let subject = || {
        subject
            .ok_or_else(|| Error::render_record(element.clone(), "frames a subject with no voxel"))
    };

    let (position, rotation) = match transform {
        PoseTransform::World { position, rotation } => {
            pose_in_frame(&TyTransformF64::IDENTITY, *position, rotation)
        }

        PoseTransform::Subject { position, rotation } => pose_in_frame(
            &TyTransformF64::from_translation(subject()?.center),
            *position,
            rotation,
        ),

        PoseTransform::Orbit {
            azimuth,
            elevation,
            distance,
        } => {
            let bounds = subject()?;

            let direction = TyVector3F64::from_azimuth_elevation(
                TyAngleUnit::Degrees.to_radians(*azimuth),
                TyAngleUnit::Degrees.to_radians(*elevation),
            );

            let distance = match (*distance, *projection) {
                (FitOrFixed::Fixed(distance), _) => distance,

                (FitOrFixed::Fit, ViewProjection::Perspective { fov }) => {
                    fit_distance(bounds, TyAngleUnit::Degrees.to_radians(fov), width, height)
                }

                // Parallel rays frame by scale alone. The camera sits one
                // scale out, past the sphere.
                (FitOrFixed::Fit, ViewProjection::Orthographic { .. }) => fit_scale(bounds),
            };

            (
                bounds.center + direction * distance,
                look_rotation(-direction),
            )
        }

        PoseTransform::Node {
            path,
            position,
            rotation,
        } => pose_in_frame(&node_frames.frame(element, path)?, *position, rotation),
    };

    let rotation = rotation
        .ok_or_else(|| Error::render_record(element.clone(), "looks at its own position"))?;

    let projection = match *projection {
        ViewProjection::Perspective { fov } => RenderProjection::Perspective {
            fov: TyAngleUnit::Degrees.to_radians(fov),
        },

        ViewProjection::Orthographic {
            scale: FitOrFixed::Fixed(scale),
        } => RenderProjection::Orthographic { scale },

        ViewProjection::Orthographic {
            scale: FitOrFixed::Fit,
        } => RenderProjection::Orthographic {
            scale: fit_scale(subject()?),
        },
    };

    Ok(RenderView {
        pose: TyPoseF64::new(position, rotation),
        projection,
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        Error, Result,
        operations::object::{
            FitOrFixed, NodeFrames, PoseTransform, RenderElement, Rotation, ViewProjection,
            resolve_view,
        },
    };
    use ty_math::{
        TyAngleUnit, TyBoundsF64, TyQuaternionExt, TyQuaternionF64, TyTransformF64, TyVector3Ext,
        TyVector3F64,
    };
    use voxcore::{VoxHierarchyNode, VoxMain};
    use voxrender::{FIT_MARGIN, RenderProjection, RenderView, fit_scale};

    fn element() -> RenderElement {
        RenderElement::ViewTransform {
            name: "hero".to_owned(),
        }
    }

    fn subject() -> TyBoundsF64 {
        TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE)
    }

    fn no_frames() -> NodeFrames {
        NodeFrames::new(&VoxMain::<()>::default(), 1.0)
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn a_world_pose_passes_through_and_the_field_of_view_turns_to_radians() {
        let rotation = TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 0.5);
        let view = resolve_view(
            &element(),
            &PoseTransform::World {
                position: TyVector3F64::new(4.0, 5.0, 6.0),
                rotation: Rotation::Quaternion { value: rotation },
            },
            &ViewProjection::Perspective { fov: 90.0 },
            None,
            &no_frames(),
            10,
            10,
        )
        .unwrap();

        assert_eq!(view.pose.position, TyVector3F64::new(4.0, 5.0, 6.0));
        assert_eq!(view.pose.rotation, rotation);
        assert_eq!(
            view.projection,
            RenderProjection::Perspective {
                fov: 90f64.to_radians()
            }
        );
    }

    #[test]
    fn an_orbit_fits_the_sphere_and_faces_the_center() {
        let view = resolve_view(
            &element(),
            &PoseTransform::Orbit {
                azimuth: 0.0,
                elevation: 0.0,
                distance: FitOrFixed::Fit,
            },
            &ViewProjection::Perspective { fov: 90.0 },
            Some(&subject()),
            &no_frames(),
            10,
            10,
        )
        .unwrap();

        let distance = 3f64.sqrt() * (1.0 + FIT_MARGIN) / 45f64.to_radians().sin();
        assert!(close(
            view.pose.position,
            subject().center + TyVector3F64::Z * distance
        ));
        assert!(
            view.pose
                .rotation
                .is_approximately_equal(TyQuaternionF64::IDENTITY, 1e-9)
        );

        // From above, the front of the subject lands at the bottom of the
        // image: the view's up is world -Z.
        let top = resolve_view(
            &element(),
            &PoseTransform::Orbit {
                azimuth: 0.0,
                elevation: 90.0,
                distance: FitOrFixed::Fixed(4.0),
            },
            &ViewProjection::Perspective { fov: 35.0 },
            Some(&subject()),
            &no_frames(),
            10,
            10,
        )
        .unwrap();
        assert!(close(
            top.pose.position,
            subject().center + TyVector3F64::Y * 4.0
        ));
        assert!(close(
            top.pose.rotation * -TyVector3F64::Z,
            -TyVector3F64::Y
        ));
        assert!(close(top.pose.rotation * TyVector3F64::Y, -TyVector3F64::Z));
    }

    #[test]
    fn an_orthographic_fit_sets_the_scale_and_seats_the_orbit_one_scale_out() {
        let view = resolve_view(
            &element(),
            &PoseTransform::Orbit {
                azimuth: 90.0,
                elevation: 0.0,
                distance: FitOrFixed::Fit,
            },
            &ViewProjection::Orthographic {
                scale: FitOrFixed::Fit,
            },
            Some(&subject()),
            &no_frames(),
            10,
            10,
        )
        .unwrap();

        let scale = fit_scale(&subject());
        assert_eq!(view.projection, RenderProjection::Orthographic { scale });
        assert!(close(
            view.pose.position,
            subject().center + TyVector3F64::X * scale
        ));

        let fixed = resolve_view(
            &element(),
            &PoseTransform::World {
                position: TyVector3F64::ZERO,
                rotation: Rotation::Euler {
                    value: TyVector3F64::ZERO,
                    unit: TyAngleUnit::Degrees,
                },
            },
            &ViewProjection::Orthographic {
                scale: FitOrFixed::Fixed(7.0),
            },
            None,
            &no_frames(),
            10,
            10,
        )
        .unwrap();
        assert_eq!(
            fixed.projection,
            RenderProjection::Orthographic { scale: 7.0 }
        );
    }

    #[test]
    fn a_subject_pose_offsets_from_the_center_and_looks_at_it_by_default() {
        let view = resolve_view(
            &element(),
            &PoseTransform::Subject {
                position: TyVector3F64::new(0.0, 0.0, 5.0),
                rotation: Rotation::LookAt { target: None },
            },
            &ViewProjection::Perspective { fov: 35.0 },
            Some(&subject()),
            &no_frames(),
            10,
            10,
        )
        .unwrap();

        assert_eq!(view.pose.position, TyVector3F64::new(1.0, 2.0, 8.0));
        assert!(
            view.pose
                .rotation
                .is_approximately_equal(TyQuaternionF64::IDENTITY, 1e-9)
        );
    }

    #[test]
    fn a_missing_subject_and_a_look_at_the_eye_error_on_the_element() {
        let reason = |result: Result<RenderView>| match result {
            Err(Error::RenderRecord { element, reason }) => (element, reason),
            other => panic!("{other:?}"),
        };

        assert_eq!(
            reason(resolve_view(
                &element(),
                &PoseTransform::Orbit {
                    azimuth: 0.0,
                    elevation: 0.0,
                    distance: FitOrFixed::Fit,
                },
                &ViewProjection::Perspective { fov: 35.0 },
                None,
                &no_frames(),
                10,
                10,
            )),
            (element(), "frames a subject with no voxel".to_owned())
        );

        assert_eq!(
            reason(resolve_view(
                &element(),
                &PoseTransform::World {
                    position: TyVector3F64::ZERO,
                    rotation: Rotation::LookAt { target: None },
                },
                &ViewProjection::Perspective { fov: 35.0 },
                None,
                &no_frames(),
                10,
                10,
            )),
            (element(), "looks at its own position".to_owned())
        );

        assert_eq!(
            reason(resolve_view(
                &element(),
                &PoseTransform::World {
                    position: TyVector3F64::ZERO,
                    rotation: Rotation::Angles {
                        azimuth: 0.0,
                        elevation: 0.0,
                    },
                },
                &ViewProjection::Orthographic {
                    scale: FitOrFixed::Fit,
                },
                None,
                &no_frames(),
                10,
                10,
            )),
            (element(), "frames a subject with no voxel".to_owned())
        );

        assert!(close(
            TyVector3F64::from_azimuth_elevation(0.0, 0.0),
            TyVector3F64::Z
        ));
    }

    #[test]
    fn a_node_pose_rides_the_path_s_world_transform_scale_included() {
        // The node turns its +Z to world +X, doubles, and sits at x = 10.
        let mut main: VoxMain = VoxMain::default();
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "player".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(10.0, 0.0, 0.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
                    TyVector3F64::splat(2.0),
                ),
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let view = resolve_view(
            &element(),
            &PoseTransform::Node {
                path: "player".to_owned(),
                position: TyVector3F64::new(0.0, 1.0, 5.0),
                rotation: Rotation::LookAt {
                    target: Some(TyVector3F64::Y),
                },
            },
            &ViewProjection::Perspective { fov: 35.0 },
            None,
            &NodeFrames::new(&main, 0.5),
            10,
            10,
        )
        .unwrap();

        // The voxel size halves the node's position alone. The offset doubles
        // under the node's scale and turns with it: 5 along +Z lands 10 along
        // +X. The look back at the node's axis turns the same way.
        assert!(close(view.pose.position, TyVector3F64::new(15.0, 2.0, 0.0)));
        assert!(close(
            view.pose.rotation * -TyVector3F64::Z,
            -TyVector3F64::X
        ));
        assert!(close(view.pose.rotation * TyVector3F64::Y, TyVector3F64::Y));

        let unmatched = resolve_view(
            &element(),
            &PoseTransform::Node {
                path: "nobody".to_owned(),
                position: TyVector3F64::ZERO,
                rotation: Rotation::Angles {
                    azimuth: 0.0,
                    elevation: 0.0,
                },
            },
            &ViewProjection::Perspective { fov: 35.0 },
            None,
            &NodeFrames::new(&main, 1.0),
            10,
            10,
        );
        assert!(matches!(
            unmatched,
            Err(Error::RenderRecord { reason, .. }) if reason.contains("matches no node path")
        ));
    }
}
