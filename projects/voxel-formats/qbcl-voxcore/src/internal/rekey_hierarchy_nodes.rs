use branded_id::U32Id;
use std::collections::BTreeMap;
use voxcore::{BVoxHierarchyNode, Error as VoxError, Result as VoxResult, VoxGcRemap};

/// `nodes` rekeyed through `remap` after a gc. An entry whose node the remap
/// released is an ext out of step, reported as an ext error naming `format`.
pub(crate) fn rekey_hierarchy_nodes<T>(
    format: &str,
    nodes: BTreeMap<U32Id<BVoxHierarchyNode>, T>,
    remap: &VoxGcRemap,
) -> VoxResult<BTreeMap<U32Id<BVoxHierarchyNode>, T>> {
    nodes
        .into_iter()
        .map(|(node_id, entry)| {
            let Some(new_id) = remap.hierarchy_nodes.new_id(node_id) else {
                return Err(VoxError::Ext {
                    reason: format!(
                        "{format} ext has an entry for released node {}",
                        node_id.to_u32()
                    ),
                });
            };
            Ok((new_id, entry))
        })
        .collect()
}
