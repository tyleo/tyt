use crate::VMaxExtNode;
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode};

/// One scene node to emit and the Voxel Max provenance that places it: the
/// voxcore node supplies the local transform, the ext supplies the id and
/// anchors, and the scene supplies the parent.
pub struct Placement<'a> {
    pub node_id: U32Id<BVoxHierarchyNode>,
    pub node: &'a VoxHierarchyNode,
    pub ext: &'a VMaxExtNode,
    pub parent_id: Option<String>,
}
