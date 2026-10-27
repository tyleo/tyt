use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyQuaternionExt, TyQuaternionF64, UNIT_ROTATION_TOLERANCE};
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

/// Sets the transform rotation of each node in `node_ids`. `rotation` is
/// stored as given. Errors, changing nothing, when an id is not one of the
/// main's nodes or `rotation` is off unit length by more than
/// [`UNIT_ROTATION_TOLERANCE`].
pub fn set_node_rotations<T: VoxExt>(
    main: &mut VoxMain<T>,
    node_ids: &[U32Id<BVoxHierarchyNode>],
    rotation: TyQuaternionF64,
) -> Result<()> {
    if !rotation.is_normalized_within(UNIT_ROTATION_TOLERANCE) {
        return Err(Error::invalid(format!(
            "rotation {rotation} has length {} and is not a unit quaternion",
            rotation.length()
        )));
    }

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

        transform.rotation = rotation;

        main.set_hierarchy_node_transform(node_id, transform)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::set_node_rotations, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// A node `a` at `(1, 2, 3)`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let a = VoxHierarchyNode {
            name: "a".to_owned(),
            transform: TyTransformF64 {
                position: TyVector3F64::new(1.0, 2.0, 3.0),
                ..Default::default()
            },
            ..Default::default()
        };

        main.retain_hierarchy_node(a).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn sets_the_rotation_and_keeps_the_rest_of_the_transform() {
        let mut main = scene();

        let rotation = TyQuaternionF64::from_rotation_z(0.5);

        set_node_rotations(&mut main, &[U32Id::from_u32(0)], rotation).unwrap();

        let a = main.hierarchy_node(U32Id::from_u32(0)).unwrap();

        assert_eq!(a.transform.rotation, rotation);
        assert_eq!(a.transform.position, TyVector3F64::new(1.0, 2.0, 3.0));
        assert_eq!(HookRecorder::events(&main), ["node 0 transform set"]);
    }

    #[test]
    fn a_rotation_within_the_tolerance_is_stored_as_given() {
        let mut main = scene();

        let rotation = TyQuaternionF64::from_xyzw(0.0, 0.0, 0.0, 1.0 + 1e-7);

        set_node_rotations(&mut main, &[U32Id::from_u32(0)], rotation).unwrap();

        assert_eq!(
            main.hierarchy_node(U32Id::from_u32(0))
                .unwrap()
                .transform
                .rotation,
            rotation
        );
    }

    #[test]
    fn a_non_unit_rotation_is_an_error_that_changes_nothing() {
        let mut main = scene();

        for rotation in [
            TyQuaternionF64::from_xyzw(0.0, 0.0, 0.0, 1.01),
            TyQuaternionF64::from_xyzw(0.0, f64::NAN, 0.0, 1.0),
        ] {
            assert!(set_node_rotations(&mut main, &[U32Id::from_u32(0)], rotation).is_err());
        }

        assert!(HookRecorder::events(&main).is_empty());
    }
}
