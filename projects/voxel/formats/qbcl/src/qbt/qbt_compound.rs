use crate::qbt::{QbtMatrix, QbtNode};

/// A `.qbt` compound node: a [`QbtMatrix`] whose grid is a baked merge of its
/// child nodes, so a viewer that does not descend the tree can still draw the
/// whole subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QbtCompound {
    /// The compound's own grid and transform: the baked merge of its children.
    pub matrix: QbtMatrix,

    /// Child nodes, in stored order.
    pub children: Vec<QbtNode>,
}
