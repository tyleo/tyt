use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{BVoxHierarchyNode, BVoxObject, VoxExt, VoxMain};

/// A hierarchy-node id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id.
type ObjectId = U32Id<BVoxObject>;

/// The hierarchy nodes of `main` whose subtree places an object in
/// `object_ids`.
pub fn placing_nodes<T: VoxExt>(
    main: &VoxMain<T>,
    object_ids: &HashSet<ObjectId>,
) -> HashSet<NodeId> {
    let mut memo = HashMap::with_capacity(main.hierarchy_node_count());

    for (node_id, _) in main.iter_hierarchy_nodes() {
        places(main, node_id, object_ids, &mut memo);
    }

    memo.into_iter()
        .filter(|(_, places)| *places)
        .map(|(node_id, _)| node_id)
        .collect()
}

/// Whether `node_id`'s subtree places an object in `object_ids`, memoized in
/// `memo` so a node shared across the DAG is walked once.
fn places<T: VoxExt>(
    main: &VoxMain<T>,
    node_id: NodeId,
    object_ids: &HashSet<ObjectId>,
    memo: &mut HashMap<NodeId, bool>,
) -> bool {
    if let Some(&places) = memo.get(&node_id) {
        return places;
    }

    let node = main
        .hierarchy_node(node_id)
        .expect("a walked node is one of the main's");

    let placing = node
        .child_object_ids
        .iter()
        .any(|object_id| object_ids.contains(object_id))
        || node
            .child_node_ids
            .iter()
            .any(|&child_id| places(main, child_id, object_ids, memo));

    memo.insert(node_id, placing);

    placing
}

#[cfg(test)]
mod tests {
    use crate::utilities::placing_nodes;
    use branded_id::U32Id;
    use std::collections::HashSet;
    use ty_math::TyVector3U32;
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject};

    #[test]
    fn a_node_places_through_its_descendants_and_a_node_reaching_nothing_does_not() {
        let mut main: VoxMain = VoxMain::default();
        let a = main
            .retain_object(VoxObject::new("a".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();
        let b = main
            .retain_object(VoxObject::new("b".to_owned(), TyVector3U32::ONE).unwrap())
            .unwrap();
        let leaf = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![a],
                ..Default::default()
            })
            .unwrap();
        let root = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_node_ids: vec![leaf],
                ..Default::default()
            })
            .unwrap();
        let other = main
            .retain_hierarchy_node(VoxHierarchyNode {
                child_object_ids: vec![b],
                ..Default::default()
            })
            .unwrap();

        let placing = placing_nodes(&main, &HashSet::from([a]));
        assert_eq!(placing, HashSet::from([leaf, root]));
        assert!(!placing.contains(&other));

        assert!(placing_nodes(&main, &HashSet::from([U32Id::from_u32(7)])).is_empty());
    }
}
