use crate::qbcl::{QbclCompound, QbclMatrix, QbclModel};

/// The body of a [`QbclNode`](crate::qbcl::QbclNode), one variant per node type
/// id. The node's name and editor flags are common to every type and live on the
/// [`QbclNode`](crate::qbcl::QbclNode) itself.
#[derive(Clone, Debug, PartialEq)]
pub enum QbclNodeBody {
    /// A matrix node (type id `0`): a single voxel grid.
    Matrix(QbclMatrix),

    /// A model node (type id `1`): groups child nodes.
    Model(QbclModel),

    /// A compound node (type id `2`): a baked voxel grid plus child nodes.
    Compound(QbclCompound),
}

impl Default for QbclNodeBody {
    fn default() -> Self {
        QbclNodeBody::Model(QbclModel::default())
    }
}
