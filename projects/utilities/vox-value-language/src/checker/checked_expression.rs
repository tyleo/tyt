use crate::{Type, checker::CheckedNode};

/// An expression with its type settled, the input to `eval_expression`.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedExpression {
    pub(crate) root: CheckedNode,
}

impl CheckedExpression {
    /// The expression's type.
    pub fn to_type(&self) -> Type {
        self.root.output
    }
}
