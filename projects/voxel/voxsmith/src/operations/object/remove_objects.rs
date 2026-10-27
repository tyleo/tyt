use crate::Result;
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxExt, VoxMain};

type NodeId = U32Id<BVoxHierarchyNode>;

type ObjectId = U32Id<BVoxObject>;

/// Detaches each object in `object_ids` from every parent and releases it,
/// then releases each node the removal left childless and repeats the check
/// on that node's parents. Nodes that were already childless stay, as do
/// palettes and value pools. The releases leave holes until [`VoxMain::gc`]
/// renumbers. Errors, changing nothing, when an id is not one of the main's
/// objects.
pub fn remove_objects<T: VoxExt>(main: &mut VoxMain<T>, object_ids: &[ObjectId]) -> Result<()> {
    for &object_id in object_ids {
        if main.object(object_id).is_none() {
            return Err(VoxError::UnknownObject { object_id }.into());
        }
    }

    let removed: HashSet<ObjectId> = object_ids.iter().copied().collect();

    let mut children: HashMap<NodeId, (Vec<NodeId>, Vec<ObjectId>)> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, node)| {
            (
                node_id,
                (node.child_node_ids.clone(), node.child_object_ids.clone()),
            )
        })
        .collect();

    let mut root_ids = main.root_hierarchy_node_ids().to_vec();

    let mut pending: Vec<NodeId> = Vec::new();

    for (&node_id, (child_node_ids, child_object_ids)) in &mut children {
        let before = child_object_ids.len();

        child_object_ids.retain(|object_id| !removed.contains(object_id));

        if child_object_ids.len() < before
            && child_node_ids.is_empty()
            && child_object_ids.is_empty()
        {
            pending.push(node_id);
        }
    }

    // A released node leaves its parents. A parent it leaves childless is
    // released too.
    let mut released: HashSet<NodeId> = HashSet::new();

    while let Some(node_id) = pending.pop() {
        if !released.insert(node_id) {
            continue;
        }

        root_ids.retain(|&root_id| root_id != node_id);

        for (&parent_id, (child_node_ids, child_object_ids)) in &mut children {
            let before = child_node_ids.len();

            child_node_ids.retain(|&child_id| child_id != node_id);

            if child_node_ids.len() < before
                && child_node_ids.is_empty()
                && child_object_ids.is_empty()
            {
                pending.push(parent_id);
            }
        }
    }

    if root_ids != main.root_hierarchy_node_ids() {
        main.set_root_hierarchy_node_ids(root_ids)?;
    }

    // Children are set in listing order so the hooks fire deterministically.
    let node_ids: Vec<NodeId> = main
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .collect();

    for &node_id in &node_ids {
        let (child_node_ids, child_object_ids) = children
            .remove(&node_id)
            .expect("children holds every node");

        let node = main
            .hierarchy_node(node_id)
            .expect("node_ids lists the main's own nodes");

        if child_node_ids != node.child_node_ids || child_object_ids != node.child_object_ids {
            main.set_hierarchy_node_children(node_id, child_node_ids, child_object_ids)?;
        }
    }

    for &node_id in &node_ids {
        if released.contains(&node_id) {
            main.release_hierarchy_node(node_id)?;
        }
    }

    let object_ids: Vec<ObjectId> = main
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .filter(|object_id| removed.contains(object_id))
        .collect();

    for object_id in object_ids {
        main.release_object(object_id)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        operations::object::remove_objects::{NodeId, ObjectId, remove_objects},
        test_utilities::HookRecorder,
    };
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxExt, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette};

    fn object_id(main: &mut VoxMain, name: &str) -> ObjectId {
        let object = VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        main.retain_object(object).unwrap()
    }

    fn node_id(
        main: &mut VoxMain,
        name: &str,
        child_node_ids: Vec<NodeId>,
        child_object_ids: Vec<ObjectId>,
    ) -> NodeId {
        let node = VoxHierarchyNode {
            name: name.to_owned(),
            child_node_ids,
            child_object_ids,
            ..Default::default()
        };

        main.retain_hierarchy_node(node).unwrap()
    }

    fn object_names<T: VoxExt>(main: &VoxMain<T>) -> Vec<&str> {
        main.iter_objects()
            .map(|(_, object)| object.name())
            .collect()
    }

    fn node_names<T: VoxExt>(main: &VoxMain<T>) -> Vec<&str> {
        main.iter_hierarchy_nodes()
            .map(|(_, node)| node.name.as_str())
            .collect()
    }

    fn root_names<T: VoxExt>(main: &VoxMain<T>) -> Vec<&str> {
        main.root_hierarchy_node_ids()
            .iter()
            .map(|&root_id| main.hierarchy_node(root_id).unwrap().name.as_str())
            .collect()
    }

    /// `root` places `a` and holds `group`, which places `b`. `other` places
    /// `c`. `empty` is a root with no children.
    fn scene() -> (VoxMain<HookRecorder>, [ObjectId; 3]) {
        let mut main: VoxMain = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");
        let c_id = object_id(&mut main, "c");

        let group_id = node_id(&mut main, "group", vec![], vec![b_id]);
        let root_id = node_id(&mut main, "root", vec![group_id], vec![a_id]);
        let other_id = node_id(&mut main, "other", vec![], vec![c_id]);
        let empty_id = node_id(&mut main, "empty", vec![], vec![]);

        for root_id in [root_id, other_id, empty_id] {
            main.push_root_hierarchy_node_id(root_id).unwrap();
        }

        (main.put_ext(HookRecorder::default()), [a_id, b_id, c_id])
    }

    #[test]
    fn a_removed_object_leaves_its_parent_when_the_parent_keeps_children() {
        let (mut main, [a_id, ..]) = scene();

        remove_objects(&mut main, &[a_id]).unwrap();

        assert_eq!(object_names(&main), ["b", "c"]);
        assert_eq!(node_names(&main), ["group", "root", "other", "empty"]);
        assert_eq!(
            HookRecorder::events(&main),
            ["node 1 children set", "object 0 released"]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_childless_node_goes_and_the_check_climbs_to_its_parents() {
        let (mut main, [a_id, b_id, _]) = scene();

        remove_objects(&mut main, &[a_id, b_id]).unwrap();

        // `group` loses `b` and goes, which leaves `root` childless too.
        // `empty` was already childless and stays.
        assert_eq!(object_names(&main), ["c"]);
        assert_eq!(node_names(&main), ["other", "empty"]);
        assert_eq!(root_names(&main), ["other", "empty"]);
        assert_eq!(
            HookRecorder::events(&main),
            [
                "roots set",
                "node 0 children set",
                "node 1 children set",
                "node 0 released",
                "node 1 released",
                "object 0 released",
                "object 1 released",
            ]
        );
        main.validate().unwrap();
    }

    #[test]
    fn a_shared_node_leaves_every_parent() {
        let mut main: VoxMain = VoxMain::default();

        let a_id = object_id(&mut main, "a");
        let b_id = object_id(&mut main, "b");

        // `shared` places `a` under both `left` and `right`. `right` also
        // places `b`.
        let shared_id = node_id(&mut main, "shared", vec![], vec![a_id]);
        let left_id = node_id(&mut main, "left", vec![shared_id], vec![]);
        let right_id = node_id(&mut main, "right", vec![shared_id], vec![b_id]);
        main.push_root_hierarchy_node_id(left_id).unwrap();
        main.push_root_hierarchy_node_id(right_id).unwrap();

        remove_objects(&mut main, &[a_id]).unwrap();

        assert_eq!(object_names(&main), ["b"]);
        assert_eq!(node_names(&main), ["right"]);
        assert_eq!(root_names(&main), ["right"]);
        main.validate().unwrap();
    }

    #[test]
    fn an_unplaced_object_is_released_and_palettes_stay() {
        let mut main: VoxMain = VoxMain::default();
        main.retain_palette(VoxPalette::default()).unwrap();

        let a_id = object_id(&mut main, "a");

        remove_objects(&mut main, &[a_id]).unwrap();

        assert_eq!(main.object_count(), 0);
        assert_eq!(main.palette_count(), 1);
        main.validate().unwrap();
    }

    #[test]
    fn an_unknown_object_is_an_error_that_changes_nothing() {
        let (mut main, [a_id, ..]) = scene();

        assert!(remove_objects(&mut main, &[a_id, U32Id::from_u32(99)]).is_err());

        assert_eq!(object_names(&main), ["a", "b", "c"]);
        assert!(HookRecorder::events(&main).is_empty());
    }
}
