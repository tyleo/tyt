use std::fmt::{Display, Formatter, Result as FmtResult};

/// A prefix operator.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum UnaryOperator {
    Negate,
    Not,
}

impl Display for UnaryOperator {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        let symbol = match self {
            UnaryOperator::Negate => "-",
            UnaryOperator::Not => "!",
        };

        formatter.write_str(symbol)
    }
}
