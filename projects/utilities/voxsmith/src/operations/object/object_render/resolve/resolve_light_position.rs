use crate::{
    Error, Result,
    operations::object::{NodeFrames, PositionTransform, RenderElement},
};
use ty_math::{TyAngleUnit, TyBoundsF64, TyPoseF64, TyVector3Ext, TyVector3F64};

/// Resolves a point light's `transform` to a world position. Errors if a
/// `subject` or `orbit` frame has no subject, or a `node` frame's glob
/// matches no node path or several.
///
/// # Arguments
/// * `element` - the light, which errors report.
/// * `subject` - the world bounds of the view's subject, or `None` when it
///   has no voxel.
/// * `view` - the resolved pose of the view being rendered.
pub fn resolve_light_position(
    element: &RenderElement,
    transform: &PositionTransform,
    subject: Option<&TyBoundsF64>,
    node_frames: &NodeFrames,
    view: &TyPoseF64,
) -> Result<TyVector3F64> {
    let subject = || {
        subject
            .ok_or_else(|| Error::render_record(element.clone(), "frames a subject with no voxel"))
    };

    Ok(match transform {
        PositionTransform::World { position } => *position,

        PositionTransform::Subject { position } => subject()?.center + *position,

        PositionTransform::Camera { position } => view.position + view.rotation * *position,

        PositionTransform::Orbit {
            azimuth,
            elevation,
            distance,
        } => {
            subject()?.center
                + TyVector3F64::from_azimuth_elevation(
                    TyAngleUnit::Degrees.to_radians(*azimuth),
                    TyAngleUnit::Degrees.to_radians(*elevation),
                ) * *distance
        }

        PositionTransform::Node { path, position } => {
            node_frames.frame(element, path)?.transform_point(*position)
        }
    })
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{
        NodeFrames, PositionTransform, RenderElement, resolve_light_position,
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
    fn each_frame_places_the_light() {
        let subject = TyBoundsF64::new(TyVector3F64::new(1.0, 2.0, 3.0), TyVector3F64::ONE);
        let view = TyPoseF64::new(
            TyVector3F64::new(0.0, 0.0, 10.0),
            TyQuaternionF64::from_axis_angle(TyVector3F64::Y, 90f64.to_radians()),
        );

        let mut main: VoxMain = VoxMain::default();
        let lamp_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "lamp".to_owned(),
                transform: TyTransformF64::new(
                    TyVector3F64::new(0.0, 4.0, 0.0),
                    TyQuaternionF64::IDENTITY,
                    TyVector3F64::splat(3.0),
                ),
                ..Default::default()
            })
            .unwrap();
        main.push_root_hierarchy_node_id(lamp_id).unwrap();
        let frames = NodeFrames::new(&main, 1.0);

        let place = |transform| {
            resolve_light_position(&element(), &transform, Some(&subject), &frames, &view)
        };

        assert_eq!(
            place(PositionTransform::World {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::X
        );
        assert_eq!(
            place(PositionTransform::Subject {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::new(2.0, 2.0, 3.0)
        );
        // The camera's +X is world -Z after a quarter turn about Y.
        assert!(close(
            place(PositionTransform::Camera {
                position: TyVector3F64::X
            })
            .unwrap(),
            TyVector3F64::new(0.0, 0.0, 9.0)
        ));
        assert!(close(
            place(PositionTransform::Orbit {
                azimuth: 90.0,
                elevation: 0.0,
                distance: 4.0,
            })
            .unwrap(),
            TyVector3F64::new(5.0, 2.0, 3.0)
        ));
        // The node's scale triples the offset.
        assert!(close(
            place(PositionTransform::Node {
                path: "lamp".to_owned(),
                position: TyVector3F64::X,
            })
            .unwrap(),
            TyVector3F64::new(3.0, 4.0, 0.0)
        ));

        assert!(
            resolve_light_position(
                &element(),
                &PositionTransform::Subject {
                    position: TyVector3F64::X
                },
                None,
                &frames,
                &view
            )
            .is_err()
        );
        assert!(
            place(PositionTransform::Node {
                path: "torch".to_owned(),
                position: TyVector3F64::X,
            })
            .is_err()
        );
    }
}
