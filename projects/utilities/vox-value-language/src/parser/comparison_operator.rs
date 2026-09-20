use std::fmt::{Display, Formatter, Result as FmtResult};

/// A comparison operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum ComparisonOperator {
    Equal,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    NotEqual,
}

impl ComparisonOperator {
    /// Whether the operator is `==` or `!=`, the pair strings take.
    pub(crate) fn is_equality(self) -> bool {
        matches!(
            self,
            ComparisonOperator::Equal | ComparisonOperator::NotEqual
        )
    }
}

impl Display for ComparisonOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let symbol = match self {
            ComparisonOperator::Equal => "==",
            ComparisonOperator::Greater => ">",
            ComparisonOperator::GreaterEqual => ">=",
            ComparisonOperator::Less => "<",
            ComparisonOperator::LessEqual => "<=",
            ComparisonOperator::NotEqual => "!=",
        };

        formatter.write_str(symbol)
    }
}
