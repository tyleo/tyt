use std::fmt::{Display, Formatter, Result as FmtResult};

/// A boolean operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum LogicalOperator {
    And,
    Or,
    Xor,
}

impl Display for LogicalOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let symbol = match self {
            LogicalOperator::And => "&&",
            LogicalOperator::Or => "||",
            LogicalOperator::Xor => "^",
        };

        formatter.write_str(symbol)
    }
}
