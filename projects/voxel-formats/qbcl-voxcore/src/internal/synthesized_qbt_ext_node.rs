use crate::QbtExtNode;
use qbcl::qbt::QbtMatrix;
use voxcore::VoxHierarchyNode;

/// The entry a node takes when nothing loaded one: a matrix entry for a node
/// placing one object and nothing else, a compound entry for a node placing
/// an object beside children or further objects, and a model entry for any
/// other node. A grid entry pivots at the grid origin at unit local scale.
pub(crate) fn synthesized_qbt_ext_node(node: &VoxHierarchyNode) -> QbtExtNode {
    let [_, extras @ ..] = node.child_object_ids.as_slice() else {
        return QbtExtNode::Model;
    };
    let local_scale = [1, 1, 1];
    let pivot = QbtMatrix::default().pivot;

    if extras.is_empty() && node.child_node_ids.is_empty() {
        return QbtExtNode::Matrix { local_scale, pivot };
    }

    QbtExtNode::Compound { local_scale, pivot }
}
