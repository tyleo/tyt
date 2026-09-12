use crate::{QbclExtNode, QbclExtNodeBody};
use qbcl::qbcl::{QbclMatrix, QbclModel};
use voxcore::VoxHierarchyNode;

/// The entry a node takes when nothing loaded one, with the editor defaults:
/// a matrix body for a node placing one object and nothing else, a compound
/// body for a node placing an object beside children or further objects, and
/// a model body with the default transform chunk for any other node. A grid
/// body pivots at the grid origin.
pub(crate) fn synthesized_qbcl_ext_node(node: &VoxHierarchyNode) -> QbclExtNode {
    let body = match node.child_object_ids.as_slice() {
        [] => QbclExtNodeBody::Model {
            transform: QbclModel::DEFAULT_TRANSFORM.to_vec(),
        },
        [_, extras @ ..] => {
            let pivot = QbclMatrix::default().pivot;

            if extras.is_empty() && node.child_node_ids.is_empty() {
                QbclExtNodeBody::Matrix { pivot }
            } else {
                QbclExtNodeBody::Compound { pivot }
            }
        }
    };

    QbclExtNode {
        body,
        ..QbclExtNode::default()
    }
}
