use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, VoxExt, VoxMain};

/// The name of the node `node_id` that the caller has checked `main` holds.
pub fn node_name<T: VoxExt>(main: &VoxMain<T>, node_id: U32Id<BVoxHierarchyNode>) -> &str {
    &main
        .hierarchy_node(node_id)
        .expect("the caller checks node_id")
        .name
}
