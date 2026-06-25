use crate::qbt::QbtNode;

/// A `.qbt` model node: an inner scene-tree node grouping child nodes. The root
/// is conventionally a model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct QbtModel {
    /// Child nodes, in stored order.
    pub children: Vec<QbtNode>,
}
