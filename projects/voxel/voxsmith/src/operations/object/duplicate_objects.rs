use crate::{Result, operations::object::link_objects};
use branded_id::U32Id;
use std::collections::HashMap;
use voxcore::{BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

type ObjectId = U32Id<BVoxObject>;

/// Appends a copy of each object in `object_ids`, with the original's name and
/// palettes, and returns the copies' ids. `parent_id` places every copy when
/// given. Otherwise each node that places an original also places its copy
/// after its existing children. A copy of an unplaced original stays unplaced.
/// Errors, changing nothing, when an id is not one of the main's.
pub fn duplicate_objects<T: VoxExt>(
    main: &mut VoxMain<T>,
    object_ids: &[ObjectId],
    parent_id: Option<NodeId>,
) -> Result<Vec<ObjectId>> {
    for &object_id in object_ids {
        if main.object(object_id).is_none() {
            return Err(VoxError::UnknownObject { object_id }.into());
        }
    }

    if let Some(parent_id) = parent_id
        && main.hierarchy_node(parent_id).is_none()
    {
        return Err(VoxError::UnknownHierarchyNode { node_id: parent_id }.into());
    }

    let mut copy_ids = Vec::with_capacity(object_ids.len());

    for &object_id in object_ids {
        let copy = main
            .object(object_id)
            .expect("object_ids are checked above")
            .clone();

        copy_ids.push(main.retain_object(copy)?);
    }

    if let Some(parent_id) = parent_id {
        link_objects(main, &copy_ids, parent_id)?;

        return Ok(copy_ids);
    }

    let copy_by_original: HashMap<ObjectId, ObjectId> = object_ids
        .iter()
        .copied()
        .zip(copy_ids.iter().copied())
        .collect();

    let mut changed = Vec::new();

    for (node_id, node) in main.iter_hierarchy_nodes() {
        let placed_copy_ids: Vec<ObjectId> = node
            .child_object_ids
            .iter()
            .filter_map(|object_id| copy_by_original.get(object_id).copied())
            .collect();

        if placed_copy_ids.is_empty() {
            continue;
        }

        let mut child_object_ids = node.child_object_ids.clone();
        child_object_ids.extend(placed_copy_ids);

        changed.push((node_id, node.child_node_ids.clone(), child_object_ids));
    }

    for (node_id, child_node_ids, child_object_ids) in changed {
        main.set_hierarchy_node_children(node_id, child_node_ids, child_object_ids)?;
    }

    Ok(copy_ids)
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::duplicate_objects::{NodeId, ObjectId, duplicate_objects},
        test_utilities::HookRecorder,
    };
    use branded_id::U32Id;
    use ty_math::{TyVector3I32, TyVector3U32};
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    fn node_id(main: &mut VoxMain, name: &str, child_object_ids: Vec<ObjectId>) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_object_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    fn children(main: &VoxMain<HookRecorder>, node_id: NodeId) -> &[ObjectId] {
        &main.hierarchy_node(node_id).unwrap().child_object_ids
    }

    /// `left` and `right` both place `a`. `b` is unplaced, and `a` has one
    /// live voxel at origin `(1, 2, 3)`.
    fn scene() -> (VoxMain<HookRecorder>, [ObjectId; 2], [NodeId; 2]) {
        let mut main: VoxMain = VoxMain::default();

        let mut a = VoxObject::new("a".to_owned(), TyVector3U32::splat(2)).unwrap();
        a.set_origin(TyVector3I32::new(1, 2, 3));
        a.retain_voxel(U32Id::from_u32(0), &[]).unwrap();

        let a_id = main.retain_object(a).unwrap();

        let b = VoxObject::new("b".to_owned(), TyVector3U32::splat(1)).unwrap();
        let b_id = main.retain_object(b).unwrap();

        let left_id = node_id(&mut main, "left", vec![a_id]);
        let right_id = node_id(&mut main, "right", vec![a_id]);
        main.push_root_hierarchy_node_id(left_id).unwrap();
        main.push_root_hierarchy_node_id(right_id).unwrap();

        (
            main.put_ext(HookRecorder::default()),
            [a_id, b_id],
            [left_id, right_id],
        )
    }

    #[test]
    fn every_parent_of_the_original_places_the_copy() {
        let (mut main, [a_id, b_id], [left_id, right_id]) = scene();

        let copy_ids = duplicate_objects(&mut main, &[a_id, b_id], None).unwrap();

        assert_eq!(copy_ids, [U32Id::from_u32(2), U32Id::from_u32(3)]);

        let copy = main.object(copy_ids[0]).unwrap();

        assert_eq!(copy.name(), "a");
        assert_eq!(copy.origin(), TyVector3I32::new(1, 2, 3));
        assert_eq!(copy.live_count(), 1);
        assert_eq!(children(&main, left_id), [a_id, copy_ids[0]]);
        assert_eq!(children(&main, right_id), [a_id, copy_ids[0]]);
        assert_eq!(
            HookRecorder::events(&main),
            [
                "object 2 retained",
                "object 3 retained",
                "node 0 children set",
                "node 1 children set",
            ]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_parent_places_every_copy_alone() {
        let (mut main, [a_id, b_id], [left_id, right_id]) = scene();

        let copy_ids = duplicate_objects(&mut main, &[a_id, b_id], Some(right_id)).unwrap();

        assert_eq!(children(&main, left_id), [a_id]);
        assert_eq!(children(&main, right_id), [a_id, copy_ids[0], copy_ids[1]]);
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_parent_is_an_error_that_changes_nothing() {
        let (mut main, [a_id, _], _) = scene();

        assert!(duplicate_objects(&mut main, &[a_id], Some(U32Id::from_u32(9))).is_err());
        assert_eq!(main.object_count(), 2);
        assert!(HookRecorder::events(&main).is_empty());
    }
}
