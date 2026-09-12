use crate::Result;
use branded_id::U32Id;
use std::collections::HashSet;
use ty_math::{TyTransformF64, TyVector3I32};
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode, VoxMain};

/// Reshapes the hierarchy into the tree a Qubicle scene holds, the shape the
/// `.qbt` and `.qbcl` writers rebuild exactly. A node reached along several
/// paths is cloned per extra path, and a node no root reaches is released.
/// Each node's translation becomes its world position, summed down from the
/// roots with each step rounded to whole voxels, because a Qubicle model
/// carries no placement of its own. Rotation and scale drop. An object no
/// node places gets a node named for it at the origin, after the roots. One
/// root named `root_name` then parents every root, in order, and the
/// unplaced objects' nodes after them.
pub fn fold_under_root(state: &mut VoxMain<()>, root_name: &str) -> Result<()> {
    let mut reached = HashSet::new();
    let mut child_node_ids = Vec::new();

    for root_id in state.root_hierarchy_node_ids().to_vec() {
        child_node_ids.push(visit(
            state,
            root_id,
            TyVector3I32::new(0, 0, 0),
            &mut reached,
        )?);
    }

    let unreached: Vec<_> = state
        .iter_hierarchy_nodes()
        .map(|(node_id, _)| node_id)
        .filter(|node_id| !reached.contains(node_id))
        .collect();

    // An unreached node's parents are all unreached. Unlinking them first
    // lets each release in any order.
    for &node_id in &unreached {
        let mut node = state
            .hierarchy_node(node_id)
            .expect("a listed node")
            .clone();
        node.child_node_ids.clear();
        state.set_hierarchy_node(node_id, node)?;
    }

    state.set_root_hierarchy_node_ids(Vec::new())?;

    for node_id in unreached {
        state.release_hierarchy_node(node_id)?;
    }

    let placed: HashSet<_> = state
        .iter_hierarchy_nodes()
        .flat_map(|(_, node)| node.child_object_ids.iter().copied())
        .collect();
    let unplaced: Vec<_> = state
        .iter_objects()
        .filter(|(object_id, _)| !placed.contains(object_id))
        .map(|(object_id, object)| VoxHierarchyNode {
            name: object.name().to_owned(),
            transform: TyTransformF64::default(),
            child_node_ids: Vec::new(),
            child_object_ids: vec![object_id],
        })
        .collect();
    for node in unplaced {
        child_node_ids.push(state.retain_hierarchy_node(node)?);
    }

    let root_id = state.retain_hierarchy_node(VoxHierarchyNode {
        name: root_name.to_owned(),
        transform: TyTransformF64::default(),
        child_node_ids,
        child_object_ids: Vec::new(),
    })?;
    state.set_root_hierarchy_node_ids(vec![root_id])?;

    Ok(())
}

/// Visits `node_id` along one more path under a parent at `parent`, and
/// returns the node that path places: the node itself on its first visit, a
/// clone after. The node's translation becomes its world position, and the
/// children are visited the same way.
fn visit(
    state: &mut VoxMain<()>,
    node_id: U32Id<BVoxHierarchyNode>,
    parent: TyVector3I32,
    reached: &mut HashSet<U32Id<BVoxHierarchyNode>>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let first = reached.insert(node_id);
    let mut node = state
        .hierarchy_node(node_id)
        .expect("a root or child is a listed node")
        .clone();
    let world = parent + node.transform.position.round().as_ivec3();
    node.transform = TyTransformF64::from_translation(world.as_dvec3());

    let mut child_node_ids = Vec::with_capacity(node.child_node_ids.len());
    for &child_id in &node.child_node_ids {
        child_node_ids.push(visit(state, child_id, world, reached)?);
    }
    node.child_node_ids = child_node_ids;

    if first {
        state.set_hierarchy_node(node_id, node)?;
        return Ok(node_id);
    }

    let clone_id = state.retain_hierarchy_node(node)?;
    reached.insert(clone_id);
    Ok(clone_id)
}
