use crate::qbt::{QbtCompound, QbtMatrix, QbtModel, QbtUnknownNode};

/// One node of a `.qbt` scene tree, one variant per node type id.
#[derive(Clone, Debug, PartialEq)]
pub enum QbtNode {
    /// A matrix node (type id `0`): a voxel grid.
    Matrix(QbtMatrix),

    /// A model node (type id `1`): groups child nodes.
    Model(QbtModel),

    /// A compound node (type id `2`): a baked voxel grid plus child nodes.
    Compound(QbtCompound),

    /// A node whose type id this crate does not model, preserved verbatim.
    Unknown(QbtUnknownNode),
}

impl Default for QbtNode {
    fn default() -> Self {
        QbtNode::Model(QbtModel::default())
    }
}
