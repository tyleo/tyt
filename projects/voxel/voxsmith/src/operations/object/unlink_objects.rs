use crate::{Error, Result};
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxExt, VoxMain};

/// Removes each object in `object_ids` from node `parent_id`'s children. An
/// object left with no parents stays in the document unplaced. The node stays
/// even when childless. Errors, changing nothing, when an id is not one of the
/// main's or `parent_id` does not place one of the objects.
pub fn unlink_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[U32Id<BVoxObject>],
    parent_id: U32Id<BVoxHierarchyNode>,
) -> Result<()> {
    let Some(parent) = main.hierarchy_node(parent_id) else {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    };

    let mut child_object_ids = parent.child_object_ids.clone();

    for &object_id in object_ids {
        let Some(object) = main.object(object_id) else {
            return Err(VoxError::UnknownObject { object_id }.into());
        };

        let Some(index) = child_object_ids
            .iter()
            .position(|&child_id| child_id == object_id)
        else {
            return Err(Error::invalid(format!(
                "node \"{}\" does not place object \"{}\"",
                parent.name,
                object.name()
            )));
        };

        child_object_ids.remove(index);
    }

    let child_node_ids = parent.child_node_ids.clone();

    Ok(main.set_hierarchy_node_children(parent_id, child_node_ids, child_object_ids)?)
}

#[cfg(test)]
mod tests {
    use crate::{operations::object::unlink_objects, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    /// `root` places `a`. `b` is unplaced.
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        let mut object_ids = Vec::new();

        for name in ["a", "b"] {
            let object = VoxObject::new(name.to_owned(), TyVector3U32::splat(1)).unwrap();

            object_ids.push(main.retain_object(object).unwrap());
        }

        let root = VoxHierarchyNode {
            name: "root".to_owned(),
            child_object_ids: vec![object_ids[0]],
            ..Default::default()
        };

        let root_id = main.retain_hierarchy_node(root).unwrap();
        main.push_root_hierarchy_node_id(root_id).unwrap();

        main.put_ext(HookRecorder::default())
    }

    #[test]
    fn leaves_the_object_and_the_childless_parent_in_the_document() {
        let mut main = scene();

        unlink_objects(&mut main, &[U32Id::from_u32(0)], U32Id::from_u32(0)).unwrap();

        assert!(
            main.hierarchy_node(U32Id::from_u32(0))
                .unwrap()
                .child_object_ids
                .is_empty()
        );
        assert_eq!(main.object_count(), 2);
        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(0)]);
        assert_eq!(HookRecorder::events(&main), ["node 0 children set"]);
        main.validate().unwrap();
    }

    #[test]
    fn a_missing_edge_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(
            unlink_objects(
                &mut main,
                &[U32Id::from_u32(0), U32Id::from_u32(1)],
                U32Id::from_u32(0)
            )
            .is_err()
        );
        assert!(HookRecorder::events(&main).is_empty());
    }
}
