use crate::operations::mesh::Transfer;

/// A write's value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WrittenValue {
    /// The expression the run evaluates.
    pub expression: String,

    /// The declared transfer.
    pub transfer: Transfer,
}
