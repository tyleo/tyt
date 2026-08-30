use crate::qbcl::{QbclMatrix, QbclNode};

/// A `.qbcl` compound node: a [`QbclMatrix`] grid that is a baked merge of its
/// child nodes, so a viewer that does not descend the tree can still draw the
/// whole subtree.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QbclCompound {
    /// The compound's own grid and transform: the baked merge of its children.
    pub matrix: QbclMatrix,

    /// Child nodes, in stored order.
    pub children: Vec<QbclNode>,
}
