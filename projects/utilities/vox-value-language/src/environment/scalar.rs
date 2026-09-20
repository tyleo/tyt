use std::fmt::{Display, Formatter, Result as FmtResult};

/// A component's type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Scalar {
    /// A 32-bit float.
    F32,

    /// An unsigned 8-bit integer.
    U8,

    /// An unsigned 16-bit integer.
    U16,

    /// An unsigned 32-bit integer.
    U32,

    /// A boolean, vec1 only.
    Bool,

    /// A string, vec1 only.
    String,
}

impl Scalar {
    /// Whether the scalar is a number.
    pub fn is_numeric(self) -> bool {
        !matches!(self, Scalar::Bool | Scalar::String)
    }

    /// Whether the scalar is an unsigned integer.
    pub fn is_unsigned(self) -> bool {
        matches!(self, Scalar::U8 | Scalar::U16 | Scalar::U32)
    }

    /// The type's name.
    pub fn name(self) -> &'static str {
        match self {
            Scalar::F32 => "f32",
            Scalar::U8 => "u8",
            Scalar::U16 => "u16",
            Scalar::U32 => "u32",
            Scalar::Bool => "bool",
            Scalar::String => "string",
        }
    }
}

impl Display for Scalar {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
        formatter.write_str(self.name())
    }
}
