use crate::operations::mesh::WrittenValue;

/// A vertex attribute write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AttributeWrite {
    /// A stream the document models, which fixes its encoding.
    Builtin {
        /// The modeled attribute.
        attribute: String,

        /// The expression the run evaluates.
        expression: String,
    },

    /// An underscore attribute.
    Custom {
        /// The attribute's name, underscore included.
        name: String,

        /// The written value.
        value: WrittenValue,
    },
}

impl AttributeWrite {
    /// The attribute the write lands on.
    pub fn name(&self) -> &str {
        match self {
            AttributeWrite::Builtin { attribute, .. } => attribute,
            AttributeWrite::Custom { name, .. } => name,
        }
    }
}
