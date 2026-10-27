use crate::SyntaxNode;

/// A `name = expr` statement.
#[derive(Clone, Debug, PartialEq)]
pub struct SyntaxBinding {
    /// The bound name.
    pub(crate) name: String,

    /// The bound expression.
    pub(crate) expression: SyntaxNode,
}
