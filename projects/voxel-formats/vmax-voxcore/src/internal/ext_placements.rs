use crate::{Error, Placement, Result, VMaxVoxMain};
use branded_id::U32Id;
use std::collections::BTreeMap;
use voxcore::BVoxHierarchyNode;

/// Pairs each voxcore node, in listing order, with its ext entry by id and
/// its parent's id from the scene. Voxel Max holds a tree. Errors when:
///
/// 1. a node has no entry, which means the ext is out of step
/// 2. a node has more than one parent
/// 3. a root is also a child, because a written node with a parent is no
///    longer a root
pub fn ext_placements(main: &VMaxVoxMain) -> Result<Vec<Placement<'_>>> {
    let ext = main.ext();

    let mut parent_ids: BTreeMap<U32Id<BVoxHierarchyNode>, U32Id<BVoxHierarchyNode>> =
        BTreeMap::new();
    for (parent_id, node) in main.iter_hierarchy_nodes() {
        for &child_id in &node.child_node_ids {
            if let Some(&other_id) = parent_ids.get(&child_id) {
                return Err(Error::invalid(format!(
                    "node {} has parents {} and {}, but a Voxel Max node has one parent",
                    child_id.to_u32(),
                    other_id.to_u32(),
                    parent_id.to_u32()
                )));
            }
            parent_ids.insert(child_id, parent_id);
        }
    }

    for &root_id in main.root_hierarchy_node_ids() {
        if let Some(parent_id) = parent_ids.get(&root_id) {
            return Err(Error::invalid(format!(
                "node {} is a root and a child of node {}, but a Voxel Max node with a parent \
                 is not a root",
                root_id.to_u32(),
                parent_id.to_u32()
            )));
        }
    }

    main.iter_hierarchy_nodes()
        .map(|(node_id, node)| {
            let entry = |id: U32Id<BVoxHierarchyNode>| {
                ext.hierarchy_nodes.get(&id).ok_or_else(|| {
                    Error::invalid(format!("vmax ext holds no entry for node {}", id.to_u32()))
                })
            };
            let parent_id = match parent_ids.get(&node_id) {
                Some(&parent_id) => Some(entry(parent_id)?.id.clone()),
                None => None,
            };
            Ok(Placement {
                node_id,
                node,
                ext: entry(node_id)?,
                parent_id,
            })
        })
        .collect()
}
