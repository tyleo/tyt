use crate::{
    Error, Result,
    operations::object::{NodeFrames, RenderElement, SpotTransform, look_rotation, pose_in_frame},
};
use ty_math::{TyAngleUnit, TyBoundsF64, TyPoseF64, TyTransformF64, TyVector3Ext, TyVector3F64};

/// Resolves a spot light's `transform` to a world pose. Errors if a
/// `subject` or `orbit` frame has no subject, a `node` frame's glob matches
/// no node path or several, or a look-at aims at the light's own position.
///
/// # Arguments
/// * `element` - the light, which errors report.
/// * `subject` - the world bounds of the view's subject, or `None` when it
///   has no voxel.
/// * `view` - the resolved pose of the view being rendered.
pub fn resolve_light_pose(
    element: &RenderElement,
    transform: &SpotTransform,
    subject: Option<&TyBoundsF64>,
    node_frames: &NodeFrames,
    view: &TyPoseF64,
) -> Result<TyPoseF64> {
    let subject = || {
        subject
            .ok_or_else(|| Error::render_record(element.clone(), "frames a subject with no voxel"))
    };

    let (position, rotation) = match transform {
        SpotTransform::World { position, rotation } => {
            pose_in_frame(&TyTransformF64::IDENTITY, *position, rotation)
        }

        SpotTransform::Subject { position, rotation } => pose_in_frame(
            &TyTransformF64::from_translation(subject()?.center),
            *position,
            rotation,
        ),

        SpotTransform::Camera { position, rotation } => pose_in_frame(
            &TyTransformF64::new(view.position, view.rotation, TyVector3F64::ONE),
            *position,
            rotation,
        ),

        SpotTransform::Orbit {
            azimuth,
            elevation,
            distance,
        } => {
            let direction = TyVector3F64::from_azimuth_elevation(
                TyAngleUnit::Degrees.to_radians(*azimuth),
                TyAngleUnit::Degrees.to_radians(*elevation),
            );

            (
                subject()?.center + direction * *distance,
                look_rotation(-direction),
            )
        }

        SpotTransform::Node {
            path,
            position,
            rotation,
        } => pose_in_frame(&node_frames.frame(element, path)?, *position, rotation),
    };

    let rotation = rotation
        .ok_or_else(|| Error::render_record(element.clone(), "looks at its own position"))?;

    Ok(TyPoseF64::new(position, rotation))
}

#[cfg(test)]
mod tests {
    use crate::{
        Error,
        operations::object::{
            NodeFrames, RenderElement, Rotation, SpotTransform, resolve_light_pose,
        },
    };
    use branded_id::U32Id;
    use ty_math::{TyBoundsF64, TyPoseF64, TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    fn element() -> RenderElement {
        RenderElement::LightTransform {
            light_id: U32Id::from_u32(0),
        }
    }

    fn close(a: TyVector3F64, b: TyVector3F64) -> bool {
        (a - b).length() < 1e-9
    }

    #[test]
    fn each_frame_poses_the_light() {
        let subject = TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE);

        // The view looks along +X from z = 10.
        let view = TyPoseF64::new(
            TyVector3F64::new(0.0, 0.0, 10.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, -90f64.to_radians()),
        );

        // The lamp node sits at y = 4, turns its +X to world -Z, and triples.
        let mut main: VoxMain = VoxMain::default();
        let lamp_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "lamp".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(0.0, 4.0, 0.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
                    TyVector3F64::splat(3.0),
                ),
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(lamp_id).unwrap();
        let frames = NodeFrames::new(&main, 1.0);

        let pose =
            |transform| resolve_light_pose(&element(), &transform, Some(&subject), &frames, &view);
        let ahead = |pose: &TyPoseF64| pose.rotation * -TyVector3F64::Z;

        let overhead = pose(SpotTransform::World {
            position: TyVector3F64::X,
            rotation: Rotation::Angles {
                azimuth: 0.0,
                elevation: 90.0,
            },
        })
        .unwrap();
        assert_eq!(overhead.position, TyVector3F64::X);
        assert!(close(ahead(&overhead), -TyVector3F64::Y));

        // A subject look-at with no target aims at the center.
        let facing = pose(SpotTransform::Subject {
            position: TyVector3F64::new(0.0, 0.0, 5.0),
            rotation: Rotation::LookAt { target: None },
        })
        .unwrap();
        assert_eq!(facing.position, TyVector3F64::new(1.0, 2.0, 8.0));
        assert!(close(ahead(&facing), -TyVector3F64::Z));

        // A headlight sits at the view and shines where it looks. The
        // camera's +X is world +Z after the quarter turn.
        let headlight = pose(SpotTransform::Camera {
            position: TyVector3F64::ZERO,
            rotation: Rotation::Angles {
                azimuth: 0.0,
                elevation: 0.0,
            },
        })
        .unwrap();
        assert!(close(headlight.position, view.position));
        assert!(close(ahead(&headlight), TyVector3F64::X));
        let beside = pose(SpotTransform::Camera {
            position: TyVector3F64::X,
            rotation: Rotation::Angles {
                azimuth: 0.0,
                elevation: 0.0,
            },
        })
        .unwrap();
        assert!(close(beside.position, TyVector3F64::new(0.0, 0.0, 11.0)));

        let orbit = pose(SpotTransform::Orbit {
            azimuth: 90.0,
            elevation: 0.0,
            distance: 4.0,
        })
        .unwrap();
        assert!(close(orbit.position, TyVector3F64::new(5.0, 2.0, 3.0)));
        assert!(close(ahead(&orbit), -TyVector3F64::X));

        let riding = pose(SpotTransform::Node {
            path: "lamp".to_owned(),
            position: TyVector3F64::X,
            rotation: Rotation::Angles {
                azimuth: 0.0,
                elevation: 0.0,
            },
        })
        .unwrap();
        assert!(close(riding.position, TyVector3F64::new(0.0, 4.0, -3.0)));
        assert!(close(ahead(&riding), -TyVector3F64::X));
    }

    #[test]
    fn a_missing_subject_a_self_look_at_and_an_unmatched_path_error() {
        let view = TyPoseF64::IDENTITY;
        let frames = NodeFrames::new(&VoxMain::<()>::default(), 1.0);

        let reason = |transform, subject: Option<&TyBoundsF64>| match resolve_light_pose(
            &element(),
            &transform,
            subject,
            &frames,
            &view,
        ) {
            Err(Error::RenderRecord { reason, .. }) => reason,
            other => panic!("{other:?}"),
        };

        assert_eq!(
            reason(
                SpotTransform::Orbit {
                    azimuth: 0.0,
                    elevation: 0.0,
                    distance: 1.0,
                },
                None
            ),
            "frames a subject with no voxel"
        );
        assert_eq!(
            reason(
                SpotTransform::World {
                    position: TyVector3F64::X,
                    rotation: Rotation::LookAt {
                        target: Some(TyVector3F64::X),
                    },
                },
                None
            ),
            "looks at its own position"
        );
        assert!(
            reason(
                SpotTransform::Node {
                    path: "torch".to_owned(),
                    position: TyVector3F64::ZERO,
                    rotation: Rotation::LookAt { target: None },
                },
                None
            )
            .contains("matches no node path")
        );
    }
}
