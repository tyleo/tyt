use ty_math::TyVector3I32;
use voxcore::VoxHierarchyNode;

/// A node's translation rounded to a Qubicle position's whole voxels.
pub fn rounded_translation(node: &VoxHierarchyNode) -> TyVector3I32 {
    node.transform.position.round().as_ivec3()
}
