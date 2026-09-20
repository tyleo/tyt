use std::fmt::{Display, Formatter, Result as FmtResult};

/// An arithmetic operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum BinaryOperator {
    Add,
    Divide,
    Multiply,
    Subtract,
}

impl Display for BinaryOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let symbol = match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Divide => "/",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Subtract => "-",
        };

        formatter.write_str(symbol)
    }
}
