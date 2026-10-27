use branded_id::U32Id;
use ty_math::TyTransformF64;
use voxcore::BVoxHierarchyNode;

/// One path to a hierarchy node, with the world transform the path gives
/// the node.
#[derive(Clone, Debug, PartialEq)]
pub struct NodePath {
    /// The node the path reaches.
    pub node_id: U32Id<BVoxHierarchyNode>,

    /// The chain of node names from the root, joined by `/`.
    pub path: String,

    /// The node transforms along the path composed, in voxel units.
    pub world: TyTransformF64,
}
