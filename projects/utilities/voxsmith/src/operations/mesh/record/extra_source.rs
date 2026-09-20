use crate::operations::mesh::WrittenValue;

/// An extras entry's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExtraSource {
    /// A referenced file.
    File(String),

    /// A written value.
    Value(WrittenValue),
}
