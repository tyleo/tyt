use crate::operations::object::Destination;
use vox_value_language::CheckedExpression;

/// A destination with its expression checked in the program's end scope.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedDestination {
    pub destination: Destination,

    pub expression: CheckedExpression,
}
