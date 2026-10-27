use crate::Result;
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

type ObjectId = U32Id<BVoxObject>;

/// Detaches each node in `node_ids` from every parent and the roots, then
/// releases it. Each descendant node and object left with no parent is
/// released too. A descendant still placed elsewhere or listed in the roots
/// stays. Releases leave holes until [`VoxMain::gc`] renumbers. Errors,
/// changing nothing, when an id is not one of the main's nodes.
pub fn remove_nodes<T: VoxExt>(main: &mut VoxMain<T>, node_ids: &[NodeId]) -> Result<()> {
    for &node_id in node_ids {
        if main.hierarchy_node(node_id).is_none() {
            return Err(VoxError::UnknownHierarchyNode { node_id }.into());
        }
    }

    let mut node_parent_ids: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

    let mut object_parent_ids: HashMap<ObjectId, Vec<NodeId>> = HashMap::new();

    for (node_id, node) in main.iter_hierarchy_nodes() {
        for &child_id in &node.child_node_ids {
            node_parent_ids.entry(child_id).or_default().push(node_id);
        }

        for &object_id in &node.child_object_ids {
            object_parent_ids
                .entry(object_id)
                .or_default()
                .push(node_id);
        }
    }

    let mut released_node_ids: HashSet<NodeId> = node_ids.iter().copied().collect();

    let mut pending: Vec<NodeId> = node_ids.to_vec();

    while let Some(node_id) = pending.pop() {
        let node = main
            .hierarchy_node(node_id)
            .expect("pending holds the main's own nodes");

        for &child_id in &node.child_node_ids {
            if released_node_ids.contains(&child_id)
                || main.root_hierarchy_node_ids().contains(&child_id)
            {
                continue;
            }

            let parent_ids = &node_parent_ids[&child_id];

            if parent_ids
                .iter()
                .all(|parent_id| released_node_ids.contains(parent_id))
            {
                released_node_ids.insert(child_id);

                pending.push(child_id);
            }
        }
    }

    let released_object_ids: HashSet<ObjectId> = object_parent_ids
        .iter()
        .filter(|(_, parent_ids)| {
            parent_ids
                .iter()
                .all(|parent_id| released_node_ids.contains(parent_id))
        })
        .map(|(&object_id, _)| object_id)
        .collect();

    let root_ids: Vec<NodeId> = main
        .root_hierarchy_node_ids()
        .iter()
        .copied()
        .filter(|root_id| !released_node_ids.contains(root_id))
        .collect();

    if root_ids != main.root_hierarchy_node_ids() {
        main.set_root_hierarchy_node_ids(root_ids)?;
    }

    let detached: Vec<(NodeId, Vec<NodeId>, Vec<ObjectId>)> = main
        .iter_hierarchy_nodes()
        .filter(|(node_id, node)| {
            !released_node_ids.contains(node_id)
                && node
                    .child_node_ids
                    .iter()
                    .any(|child_id| released_node_ids.contains(child_id))
        })
        .map(|(node_id, node)| {
            let child_node_ids = node
                .child_node_ids
                .iter()
                .copied()
                .filter(|child_id| !released_node_ids.contains(child_id))
                .collect();

            (node_id, child_node_ids, node.child_object_ids.clone())
        })
        .collect();

    for (node_id, child_node_ids, child_object_ids) in detached {
        main.set_hierarchy_node_children(node_id, child_node_ids, child_object_ids)?;
    }

    // voxcore refuses to release a listed node. Each pass releases, in
    // listing order, the nodes whose released parents are gone.
    let mut unreleased: Vec<NodeId> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| released_node_ids.contains(node_id))
        .collect();

    while !unreleased.is_empty() {
        let (ready, waiting): (Vec<NodeId>, Vec<NodeId>) =
            unreleased.iter().copied().partition(|node_id| {
                node_parent_ids
                    .get(node_id)
                    .is_none_or(|parent_ids| !parent_ids.iter().any(|id| unreleased.contains(id)))
            });

        for node_id in ready {
            main.release_hierarchy_node(node_id)?;
        }

        unreleased = waiting;
    }

    let object_ids: Vec<ObjectId> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .filter(|object_id| released_object_ids.contains(object_id))
        .collect();

    for object_id in object_ids {
        main.release_object(object_id)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{operations::node::remove_nodes, test_utilities::HookRecorder};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    fn node(name: &str, child_node_ids: &[u32], child_object_ids: &[u32]) -> VoxHierarchyNode {
        VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids: child_node_ids
                .iter()
                .copied()
                .map(U32Id::from_u32)
                .collect(),
            child_object_ids: child_object_ids
                .iter()
                .copied()
                .map(U32Id::from_u32)
                .collect(),
            ..Default::default()
        }
    }

    /// Objects `knob` 0, `hinge` 1, and `wall` 2. Nodes:
    ///
    /// 1. `door` 0 places `knob` and `hinge`
    /// 2. `frame` 1 places `door` and `hinge`
    /// 3. `house` 2, a root, places `frame` and `wall`
    /// 4. `garage` 3, a root, places `hinge`
    fn scene() -> VoxMain<HookRecorder> {
        let mut main: VoxMain = VoxMain::default();

        for name in ["knob", "hinge", "wall"] {
            let object = VoxObject::new(name.to_owned(), TyVector3U32::splat(1)).unwrap();

            main.retain_object(object).unwrap();
        }

        main.retain_hierarchy_node(node("door", &[], &[0, 1]))
            .unwrap();
        main.retain_hierarchy_node(node("frame", &[0], &[1]))
            .unwrap();
        main.retain_hierarchy_node(node("house", &[1], &[2]))
            .unwrap();
        main.retain_hierarchy_node(node("garage", &[], &[1]))
            .unwrap();

        main.set_root_hierarchy_node_ids(vec![U32Id::from_u32(2), U32Id::from_u32(3)])
            .unwrap();

        main.put_ext(HookRecorder::default())
    }

    fn names(main: &VoxMain<HookRecorder>) -> (Vec<&str>, Vec<&str>) {
        let node_names = main
            .iter_hierarchy_nodes()
            .map(|(_, node)| node.name.as_str())
            .collect();

        let object_names = main
            .iter_objects()
            .map(|(_, object)| object.name())
            .collect();

        (node_names, object_names)
    }

    #[test]
    fn releases_the_descendants_left_with_no_parent() {
        let mut main = scene();

        remove_nodes(&mut main, &[U32Id::from_u32(1)]).unwrap();

        // hinge stays under garage.
        assert_eq!(
            names(&main),
            (vec!["house", "garage"], vec!["hinge", "wall"])
        );
        assert_eq!(
            HookRecorder::events(&main),
            [
                "node 2 children set",
                "node 1 released",
                "node 0 released",
                "object 0 released",
            ]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_root_leaves_the_roots_and_a_rooted_descendant_stays() {
        let mut main = scene();

        let mut roots = main.root_hierarchy_node_ids().to_vec();
        roots.push(U32Id::from_u32(1));
        main.set_root_hierarchy_node_ids(roots).unwrap();

        remove_nodes(&mut main, &[U32Id::from_u32(2), U32Id::from_u32(3)]).unwrap();

        assert_eq!(names(&main), (vec!["door", "frame"], vec!["knob", "hinge"]));
        assert_eq!(main.root_hierarchy_node_ids(), [U32Id::from_u32(1)]);
        main.validate().unwrap();
    }

    #[test]
    fn a_selected_node_below_another_releases_after_it() {
        let mut main = scene();

        remove_nodes(&mut main, &[U32Id::from_u32(0), U32Id::from_u32(2)]).unwrap();

        assert_eq!(names(&main), (vec!["garage"], vec!["hinge"]));
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_node_is_an_error_that_changes_nothing() {
        let mut main = scene();

        assert!(remove_nodes(&mut main, &[U32Id::from_u32(1), U32Id::from_u32(9)]).is_err());
        assert!(HookRecorder::events(&main).is_empty());
    }
}
