use crate::{
    Type,
    checker::{CheckedNode, Reads, climbs},
};

/// An expression with its type settled, the input to `eval_expression`.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedExpression {
    pub(crate) root: CheckedNode,
}

impl CheckedExpression {
    /// The array values the expression lifts above their domain, each as
    /// an expression over the same scope: an operand paired with a higher
    /// domain, a climb function's argument, and a bound `default` name.
    pub fn climbs(&self) -> Vec<CheckedExpression> {
        let mut lifted = Vec::new();

        climbs(&self.root, &mut lifted);

        lifted
    }

    /// What the expression reads.
    pub fn reads(&self) -> Reads {
        let mut reads = Reads::default();

        reads.gather(&self.root, false);

        reads
    }

    /// The expression's type.
    pub fn to_type(&self) -> Type {
        self.root.output
    }
}
