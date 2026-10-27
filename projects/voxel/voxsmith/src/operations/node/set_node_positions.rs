use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::TyVector3F64;
use voxcore::{BVoxHierarchyNode, Error as VoxError, VoxExt, VoxMain};

/// Sets the transform position of each node in `node_ids`. Errors, changing
/// nothing, when an id is not one of the main's nodes or `position` has a
/// non-finite component.
pub fn set_node_positions<T: VoxExt>(
    main: &mut VoxMain<T>,
    node_ids: &[U32Id<BVoxHierarchyNode>],
    position: TyVector3F64,
) -> Result<()> {
    if !position.is_finite() {
        return Err(Error::invalid(format!(
            "position {position} has a non-finite component"
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

        transform.position = position;

        main.set_hierarchy_node_transform(node_id, transform)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::set_node_positions, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64};
    use voxcore::{VoxHierarchyNode, VoxMain};

    /// Nodes `a` and `b`, with `a` turned a quarter about `y`.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let a = VoxHierarchyNode {
            name: "a".to_owned(),
            transform: TyTransformF64 {
                rotation: TyQuaternionF64::from_rotation_y(1.0),
                ..Default::default()
            },
            ..Default::default()
        };

        main.retain_hierarchy_node(a).unwrap();

        let b = VoxHierarchyNode {
            name: "b".to_owned(),
            ..Default::default()
        };

        main.retain_hierarchy_node(b).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn sets_the_position_and_keeps_the_rest_of_the_transform() {
        let mut main = scene();

        let position = TyVector3F64::new(-1.5, 2.0, 0.0);

        set_node_positions(
            &mut main,
            &[U32Id::from_u32(0), U32Id::from_u32(1)],
            position,
        )
        .unwrap();

        let a = main.hierarchy_node(U32Id::from_u32(0)).unwrap();

        assert_eq!(a.transform.position, position);
        assert_eq!(a.transform.rotation, TyQuaternionF64::from_rotation_y(1.0));
        assert_eq!(
            HookRecorder::events(&main),
            ["node 0 transform set", "node 1 transform set"]
        );
    }

    #[test]
    fn a_non_finite_position_or_unknown_node_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(
            set_node_positions(
                &mut main,
                &[U32Id::from_u32(0)],
                TyVector3F64::new(0.0, f64::NAN, 0.0)
            )
            .is_err()
        );
        assert!(
            set_node_positions(
                &mut main,
                &[U32Id::from_u32(0), U32Id::from_u32(9)],
                TyVector3F64::ONE
            )
            .is_err()
        );
        assert!(HookRecorder::events(&main).is_empty());
    }
}
