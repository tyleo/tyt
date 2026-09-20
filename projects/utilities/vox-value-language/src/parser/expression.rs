use crate::parser::SyntaxNode;

/// A parsed expression, the input to `check_expression`.
#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub(crate) root: SyntaxNode,
}
