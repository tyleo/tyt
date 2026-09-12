use crate::{Error, Placement, Result, VMaxExt, VMaxVoxMain, synth_uuid, synthesized_node};
use branded_id::U32Id;
use std::collections::{HashMap, HashSet};
use voxcore::{BVoxHierarchyNode, VoxMain};

/// Pairs each voxcore node with its ext entry by listing index, the placement
/// the writer emits. A vmax-origin scene is a tree with one ext node per
/// voxcore node, so this reproduces it exactly. An entry with no id was
/// inserted by a hook for a node retained after the load, so it is filled in
/// like a synthesized node: a fresh UUID no other entry uses, its first
/// parent's id, the node's rotation, and the default anchor tokens. Errors
/// when the ext is out of step with the listings.
pub fn ext_placements(state: &VMaxVoxMain) -> Result<Vec<Placement<'_>>> {
    let vmax_ext = state.ext();
    check_alignment(state, vmax_ext)?;

    // Every id first, so a child can link to an inserted parent anywhere in
    // the listing.
    let taken: HashSet<&str> = vmax_ext
        .hierarchy_nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect();
    let mut counter = 0usize;
    let ids: Vec<String> = vmax_ext
        .hierarchy_nodes
        .iter()
        .map(|node| {
            if !node.id.is_empty() {
                return node.id.clone();
            }
            loop {
                let id = synth_uuid(counter);
                counter += 1;
                if !taken.contains(id.as_str()) {
                    return id;
                }
            }
        })
        .collect();

    let mut parent_indices: HashMap<U32Id<BVoxHierarchyNode>, usize> = HashMap::new();
    for (index, (_, node)) in state.iter_hierarchy_nodes().enumerate() {
        for &child_id in &node.child_node_ids {
            parent_indices.entry(child_id).or_insert(index);
        }
    }

    Ok(state
        .iter_hierarchy_nodes()
        .enumerate()
        .map(|(index, (node_id, node))| {
            let stored = &vmax_ext.hierarchy_nodes[index];
            let ext = if stored.id.is_empty() {
                let parent_id = parent_indices
                    .get(&node_id)
                    .map(|&parent_index| ids[parent_index].clone());
                synthesized_node(ids[index].clone(), parent_id, node)
            } else {
                stored.clone()
            };
            Placement { node_id, node, ext }
        })
        .collect())
}

/// Checks that each aligned list of the ext is as long as its listing. The
/// hooks keep them in step, so a mismatch is a malformed ext.
fn check_alignment<T>(state: &VoxMain<T>, vmax_ext: &VMaxExt) -> Result<()> {
    let lists = [
        (
            "hierarchy nodes",
            vmax_ext.hierarchy_nodes.len(),
            state.hierarchy_node_count(),
        ),
        ("palettes", vmax_ext.palettes.len(), state.palette_count()),
        (
            "object states",
            vmax_ext.object_states.len(),
            state.object_count(),
        ),
    ];
    for (list, stored, count) in lists {
        if stored != count {
            return Err(Error::invalid(format!(
                "vmax ext has {stored} {list} but the state has {count}"
            )));
        }
    }
    Ok(())
}
