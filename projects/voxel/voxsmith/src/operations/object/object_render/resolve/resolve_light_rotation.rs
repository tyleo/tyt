use crate::{
    Error, Result,
    operations::object::{
        NodeFrames, RenderElement, Rotation, RotationTransform, resolve_rotation,
    },
};
use ty_math::{TyPoseF64, TyQuaternionF64, TyVector3F64};

/// Resolves a directional light's `transform` to a world rotation. Errors if
/// a look-at has no target, because the light sits at its frame's origin, or
/// a `node` frame's glob matches no node path or several.
///
/// # Arguments
/// * `element` - the light, which errors report.
/// * `view` - the resolved pose of the view being rendered.
pub fn resolve_light_rotation(
    element: &RenderElement,
    transform: &RotationTransform,
    node_frames: &NodeFrames,
    view: &TyPoseF64,
) -> Result<TyQuaternionF64> {
    let (frame, rotation): (TyQuaternionF64, &Rotation) = match transform {
        RotationTransform::World { rotation } => (TyQuaternionF64::IDENTITY, rotation),

        RotationTransform::Camera { rotation } => (view.rotation, rotation),

        RotationTransform::Node { path, rotation } => {
            (node_frames.frame(element, path)?.rotation, rotation)
        }
    };

    let local = resolve_rotation(rotation, TyVector3F64::ZERO)
        .ok_or_else(|| Error::render_record(element.clone(), "looks at its own position"))?;

    Ok(frame * local)
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        NodeFrames, RenderElement, Rotation, RotationTransform, resolve_light_rotation,
    };
    use branded_id::U32Id;
    use ty_math::{TyPoseF64, TyQuaternionF64, TyTransformF64, TyVector3Ext, TyVector3F64};
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
    fn a_camera_frame_light_turns_with_the_view() {
        // The view looks along +X, which turns the view's -X into world -Z.
        // A light at the view's upper left comes from world -Z and +Y.
        let view = TyPoseF64::new(
            TyVector3F64::ZERO,
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, -90f64.to_radians()),
        );
        assert!(close(view.rotation * -TyVector3F64::Z, TyVector3F64::X));

        let frames = NodeFrames::new(&VoxMain::<()>::default(), 1.0);

        let rotation = resolve_light_rotation(
            &element(),
            &RotationTransform::Camera {
                rotation: Rotation::Angles {
                    azimuth: -30.0,
                    elevation: 30.0,
                },
            },
            &frames,
            &view,
        )
        .unwrap();

        let from = view.rotation
            * TyVector3F64::from_azimuth_elevation(-30f64.to_radians(), 30f64.to_radians());
        assert!(close(rotation * -TyVector3F64::Z, -from));
        assert!(from.y > 0.0 && from.z < 0.0);

        let world = resolve_light_rotation(
            &element(),
            &RotationTransform::World {
                rotation: Rotation::Angles {
                    azimuth: 0.0,
                    elevation: 90.0,
                },
            },
            &frames,
            &view,
        )
        .unwrap();
        assert!(close(world * -TyVector3F64::Z, -TyVector3F64::Y));

        assert!(
            resolve_light_rotation(
                &element(),
                &RotationTransform::World {
                    rotation: Rotation::LookAt { target: None },
                },
                &frames,
                &view,
            )
            .is_err()
        );
    }

    #[test]
    fn a_node_frame_light_turns_with_the_node() {
        // The node turns its -Z to world -X.
        let mut main: VoxMain = VoxMain::default();
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "lamp".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(3.0, 0.0, 0.0),
                    TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
                    TyVector3F64::splat(2.0),
                ),
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();

        let rotation = resolve_light_rotation(
            &element(),
            &RotationTransform::Node {
                path: "lamp".to_owned(),
                rotation: Rotation::Angles {
                    azimuth: 0.0,
                    elevation: 0.0,
                },
            },
            &NodeFrames::new(&main, 1.0),
            &TyPoseF64::IDENTITY,
        )
        .unwrap();

        assert!(close(rotation * -TyVector3F64::Z, -TyVector3F64::X));
    }
}
