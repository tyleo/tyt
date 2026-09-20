use crate::checker::CheckedNode;

/// A binding with its expression checked.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CheckedBinding {
    /// The bound name.
    pub(crate) name: String,

    /// The checked expression.
    pub(crate) expression: CheckedNode,
}
