use crate::ext::VMaxExtNode;
use branded_id::U32Id;
use voxcore::{BVoxHierarchyNode, VoxHierarchyNode};

/// One scene node to emit and the Voxel Max provenance that places it: the
/// voxcore node supplies the local transform, the ext supplies the id, parent,
/// and bounds. The lossless path pairs each voxcore node with its ext entry by
/// index; synthesis walks the hierarchy from the roots, so a node shared by
/// several parents, or one that is both a root and a child, is emitted once per
/// path the way voxcore renders it, and a node reachable from no root is
/// dropped just as voxcore never places it.
pub struct Placement<'a> {
    pub node_id: U32Id<BVoxHierarchyNode>,
    pub node: &'a VoxHierarchyNode,
    pub ext: VMaxExtNode,
}
