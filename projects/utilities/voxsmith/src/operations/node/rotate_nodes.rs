use crate::Result;
use branded_id::U32Id;
use ty_math::{TyAxis3, TyQuaternionExt};
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

/// Rotates each node in `node_ids` by `degrees` about `axis` of its parent's
/// frame on top of the node's rotation. Errors, changing nothing, when an id
/// is not one of the main's nodes.
pub fn rotate_nodes<T: VoxExt>(
    main: &mut VoxMain<T>,
    node_ids: &[U32Id<BVoxHierarchyNode>],
    axis: TyAxis3,
    degrees: f64,
) -> Result<()> {
    for &node_id in node_ids {
        if main.hierarchy_node(node_id).is_none() {
            return Err(VoxError::UnknownHierarchyNode { node_id }.into());
        }
    }

    for &node_id in node_ids {
        let mut transform = main
            .hierarchy_node(node_id)
            .expect("node_ids are checked above")
            .transform;

        transform.rotation = transform
            .rotation
            .rotate_around_axis(axis.unit_vector3_f64(), degrees.to_radians());

        main.set_hierarchy_node_transform(node_id, transform)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::rotate_nodes, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyAxis3, TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// A node `a` at `(1, 2, 3)` rotated 90 degrees about z.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let a = VoxHierarchyNode {
            name: "a".to_owned(),
            transform: TyTransformF64 {
                position: TyVector3F64::new(1.0, 2.0, 3.0),
                rotation: TyQuaternionF64::from_rotation_z(90f64.to_radians()),
                ..Default::default()
            },
            ..Default::default()
        };

        main.retain_hierarchy_node(a).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn a_rotation_applies_after_the_node_rotation_on_the_parent_axes() {
        let mut main = scene();

        rotate_nodes(&mut main, &[U32Id::from_u32(0)], TyAxis3::X, 90.0).unwrap();

        let a = main.hierarchy_node(U32Id::from_u32(0)).unwrap();

        // The rotation about z carries +x onto +y, and the rotation about x
        // then carries +y onto +z.
        assert!((a.transform.rotation * TyVector3F64::X).abs_diff_eq(TyVector3F64::Z, 1e-12));
        assert_eq!(a.transform.position, TyVector3F64::new(1.0, 2.0, 3.0));
        assert_eq!(HookRecorder::events(&main), ["node 0 transform set"]);
    }

    #[test]
    fn an_unknown_node_is_an_error_that_changes_nothing() {
        let mut main = scene();

        let node_ids = [U32Id::from_u32(0), U32Id::from_u32(7)];

        assert!(rotate_nodes(&mut main, &node_ids, TyAxis3::X, 90.0).is_err());
        assert!(HookRecorder::events(&main).is_empty());
    }
}
