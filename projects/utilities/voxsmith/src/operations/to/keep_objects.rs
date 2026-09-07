use crate::Result;
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, Error as VoxError, VoxHierarchyNode, VoxMain, ext::VoxExt,
};

/// A hierarchy-node id.
type NodeId = U32Id<BVoxHierarchyNode>;

/// An object id.
type ObjectId = U32Id<BVoxObject>;

/// Keeps only the objects in `object_ids`, releasing every other object and
/// every hierarchy node whose subtree no longer places a kept object. A
/// surviving node keeps its transform and its surviving children. Order is
/// preserved throughout: child lists, roots, and the node and object
/// listings. Palettes and value pools are untouched. The releases leave
/// holes until [`VoxMain::gc`] renumbers. Errors, changing nothing, when an
/// id is not one of the state's objects.
pub fn keep_objects<T: VoxExt>(state: &mut VoxMain<T>, object_ids: &[ObjectId]) -> Result<()> {
    for &object_id in object_ids {
        if state.object(object_id).is_none() {
            return Err(VoxError::UnknownObject { object_id }.into());
        }
    }

    let kept: HashSet<ObjectId> = object_ids.iter().copied().collect();

    let nodes: Vec<(NodeId, VoxHierarchyNode)> = state
        .iter_hierarchy_nodes()
        .map(|(node_id, node)| (node_id, node.clone()))
        .collect();

    let by_id: HashMap<NodeId, &VoxHierarchyNode> = nodes
        .iter()
        .map(|(node_id, node)| (*node_id, node))
        .collect();

    let alive = alive_nodes(&by_id, &kept);

    // A dead root has to leave the roots before it can be released.
    let root_ids: Vec<NodeId> = state
        .root_hierarchy_node_ids()
        .iter()
        .copied()
        .filter(|root_id| alive.contains(root_id))
        .collect();

    state.set_root_hierarchy_node_ids(root_ids)?;

    // A surviving node drops its dead child nodes and dropped objects. A dead
    // node is emptied because a listed child cannot be released. After this
    // pass nothing references a dead node or a dropped object.
    for (node_id, node) in &nodes {
        let mut pruned = node.clone();

        if alive.contains(node_id) {
            pruned
                .child_node_ids
                .retain(|child_id| alive.contains(child_id));

            pruned
                .child_object_ids
                .retain(|child_id| kept.contains(child_id));
        } else {
            pruned.child_node_ids.clear();
            pruned.child_object_ids.clear();
        }

        if pruned != *node {
            state.set_hierarchy_node(*node_id, pruned)?;
        }
    }

    for (node_id, _) in &nodes {
        if !alive.contains(node_id) {
            state.release_hierarchy_node(*node_id)?;
        }
    }

    let dropped: Vec<ObjectId> = state
        .iter_objects()
        .map(|(object_id, _)| object_id)
        .filter(|object_id| !kept.contains(object_id))
        .collect();

    for object_id in dropped {
        state.release_object(object_id)?;
    }

    Ok(())
}

/// The nodes whose subtree places a kept object.
fn alive_nodes(
    by_id: &HashMap<NodeId, &VoxHierarchyNode>,
    kept: &HashSet<ObjectId>,
) -> HashSet<NodeId> {
    let mut memo = HashMap::with_capacity(by_id.len());

    for &node_id in by_id.keys() {
        is_alive(node_id, by_id, kept, &mut memo);
    }

    memo.into_iter()
        .filter(|(_, alive)| *alive)
        .map(|(node_id, _)| node_id)
        .collect()
}

/// Whether `node_id`'s subtree places a kept object, memoized in `memo` so a
/// node shared across the DAG is walked once.
fn is_alive(
    node_id: NodeId,
    by_id: &HashMap<NodeId, &VoxHierarchyNode>,
    kept: &HashSet<ObjectId>,
    memo: &mut HashMap<NodeId, bool>,
) -> bool {
    if let Some(&alive) = memo.get(&node_id) {
        return alive;
    }

    let node = by_id[&node_id];

    let alive = node
        .child_object_ids
        .iter()
        .any(|object_id| kept.contains(object_id))
        || node
            .child_node_ids
            .iter()
            .any(|&child_id| is_alive(child_id, by_id, kept, memo));

    memo.insert(node_id, alive);

    alive
}

#[cfg(test)]
mod tests {
    use crate::operations::to::keep_objects::{NodeId, ObjectId, keep_objects};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{VoxHierarchyNode, VoxMain, VoxObject, VoxPalette};

    /// Adds an empty named object and returns its id.
    fn object_id(state: &mut VoxMain, name: &str) -> ObjectId {
        let object = VoxObject::new(name.to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();

        state.retain_object(object).unwrap()
    }

    /// Adds a hierarchy node placing the given child nodes and objects.
    fn node_id(
        state: &mut VoxMain,
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

        state.retain_hierarchy_node(node).unwrap()
    }

    /// The object names in listing order.
    fn object_names(state: &VoxMain) -> Vec<&str> {
        state
            .iter_objects()
            .map(|(_, object)| object.name())
            .collect()
    }

    /// Each node's name, child node names, and child object names, in listing
    /// order.
    fn node_shapes(state: &VoxMain) -> Vec<(String, Vec<String>, Vec<String>)> {
        state
            .iter_hierarchy_nodes()
            .map(|(_, node)| {
                let child_nodes = node
                    .child_node_ids
                    .iter()
                    .map(|&child_id| state.hierarchy_node(child_id).unwrap().name.clone())
                    .collect();

                let child_objects = node
                    .child_object_ids
                    .iter()
                    .map(|&child_id| state.object(child_id).unwrap().name().to_owned())
                    .collect();

                (node.name.clone(), child_nodes, child_objects)
            })
            .collect()
    }

    /// The root node names in order.
    fn root_names(state: &VoxMain) -> Vec<String> {
        state
            .root_hierarchy_node_ids()
            .iter()
            .map(|&root_id| state.hierarchy_node(root_id).unwrap().name.clone())
            .collect()
    }

    /// A `node_shapes` entry built from names.
    fn shape(
        name: &str,
        child_nodes: &[&str],
        child_objects: &[&str],
    ) -> (String, Vec<String>, Vec<String>) {
        (
            name.to_owned(),
            child_nodes.iter().map(|name| name.to_string()).collect(),
            child_objects.iter().map(|name| name.to_string()).collect(),
        )
    }

    /// Two roots. `root` places `a` and holds `group`, which places `b` and
    /// `c`. `other` places `d`. Object `e` is unplaced.
    fn scene() -> (VoxMain, Vec<ObjectId>) {
        let mut state = VoxMain::default();

        let a_id = object_id(&mut state, "a");
        let b_id = object_id(&mut state, "b");
        let c_id = object_id(&mut state, "c");
        let d_id = object_id(&mut state, "d");
        let e_id = object_id(&mut state, "e");

        let group_id = node_id(&mut state, "group", vec![], vec![b_id, c_id]);
        let root_id = node_id(&mut state, "root", vec![group_id], vec![a_id]);
        let other_id = node_id(&mut state, "other", vec![], vec![d_id]);
        state.push_root_hierarchy_node_id(root_id).unwrap();
        state.push_root_hierarchy_node_id(other_id).unwrap();

        (state, vec![a_id, b_id, c_id, d_id, e_id])
    }

    #[test]
    fn keeping_every_object_leaves_the_scene_as_it_was() {
        let (mut state, ids) = scene();

        let before = (
            node_shapes(&state),
            root_names(&state),
            object_names(&state).len(),
        );

        keep_objects(&mut state, &ids).unwrap();

        assert_eq!(
            (
                node_shapes(&state),
                root_names(&state),
                object_names(&state).len()
            ),
            before
        );
        assert_eq!(object_names(&state), ["a", "b", "c", "d", "e"]);
        state.validate().unwrap();
    }

    #[test]
    fn keeping_one_object_prunes_the_nodes_left_placing_nothing() {
        let (mut state, ids) = scene();

        keep_objects(&mut state, &[ids[2]]).unwrap();

        // Only `c` remains. `group` and `root` survive as its ancestors, their
        // other children gone. `other` is dropped with `d`.
        assert_eq!(object_names(&state), ["c"]);
        assert_eq!(
            node_shapes(&state),
            [shape("group", &[], &["c"]), shape("root", &["group"], &[])]
        );
        assert_eq!(root_names(&state), ["root"]);
        state.validate().unwrap();

        // The holes compact away for a deterministic write.
        state.gc();
        assert_eq!(state.object_count(), 1);
        assert_eq!(state.hierarchy_node_count(), 2);
        state.validate().unwrap();
    }

    #[test]
    fn a_kept_unplaced_object_survives_without_a_node() {
        let (mut state, ids) = scene();

        keep_objects(&mut state, &[ids[4]]).unwrap();

        assert_eq!(object_names(&state), ["e"]);
        assert_eq!(state.hierarchy_node_count(), 0);
        assert!(state.root_hierarchy_node_ids().is_empty());
        state.validate().unwrap();
    }

    #[test]
    fn a_shared_node_survives_through_any_parent_and_a_dead_parent_goes() {
        let mut state = VoxMain::default();

        let a_id = object_id(&mut state, "a");
        let b_id = object_id(&mut state, "b");

        // `shared` places `a` under both `left` and `right`. `right` also
        // places `b`.
        let shared_id = node_id(&mut state, "shared", vec![], vec![a_id]);
        let left_id = node_id(&mut state, "left", vec![shared_id], vec![]);
        let right_id = node_id(&mut state, "right", vec![shared_id], vec![b_id]);
        state.push_root_hierarchy_node_id(left_id).unwrap();
        state.push_root_hierarchy_node_id(right_id).unwrap();

        keep_objects(&mut state, &[a_id]).unwrap();

        assert_eq!(object_names(&state), ["a"]);
        assert_eq!(
            node_shapes(&state),
            [
                shape("shared", &[], &["a"]),
                shape("left", &["shared"], &[]),
                shape("right", &["shared"], &[]),
            ]
        );
        assert_eq!(root_names(&state), ["left", "right"]);

        // Keeping only a re-added `b`, which no node places, drops every node.
        let mut state = state;
        let b_id = object_id(&mut state, "b");
        keep_objects(&mut state, &[b_id]).unwrap();

        assert_eq!(object_names(&state), ["b"]);
        assert_eq!(state.hierarchy_node_count(), 0);
        state.validate().unwrap();
    }

    #[test]
    fn keeping_the_object_order_and_the_palettes() {
        let (mut state, ids) = scene();
        state.retain_palette(VoxPalette::default()).unwrap();

        keep_objects(&mut state, &[ids[3], ids[0]]).unwrap();

        // Objects keep the listing order, not the order given.
        assert_eq!(object_names(&state), ["a", "d"]);
        assert_eq!(state.palette_count(), 1);
        state.validate().unwrap();
    }

    #[test]
    fn an_unknown_object_is_an_error_that_changes_nothing() {
        let (mut state, ids) = scene();

        let before = node_shapes(&state);

        assert!(keep_objects(&mut state, &[ids[0], U32Id::from_u32(99)]).is_err());

        assert_eq!(node_shapes(&state), before);
        assert_eq!(object_names(&state), ["a", "b", "c", "d", "e"]);
    }
}
