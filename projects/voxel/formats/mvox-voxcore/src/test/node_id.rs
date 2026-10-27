use branded_id::U32Id;
use voxcore::BVoxHierarchyNode;

/// The hierarchy node id `index`.
pub fn node_id(index: u32) -> U32Id<BVoxHierarchyNode> {
    U32Id::from_u32(index)
}
