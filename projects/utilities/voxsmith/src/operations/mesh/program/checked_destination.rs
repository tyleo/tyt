use crate::operations::mesh::Destination;
use vox_value_language::CheckedExpression;

/// A destination with its expression checked in the program's end scope.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CheckedDestination {
    pub(crate) destination: Destination,
    pub(crate) expression: CheckedExpression,
}
