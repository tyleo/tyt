use crate::Scalar;

/// A value's entries, flattened component by component; a plain value holds
/// one entry.
#[derive(Clone, Debug, PartialEq)]
pub enum Components {
    /// `f32` components.
    F32(Vec<f32>),

    /// `u8` components.
    U8(Vec<u8>),

    /// `u16` components.
    U16(Vec<u16>),

    /// `u32` components.
    U32(Vec<u32>),

    /// `bool` components.
    Bool(Vec<bool>),

    /// `String` components.
    String(Vec<String>),
}

impl Components {
    /// Whether there are no components.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The component count.
    pub fn len(&self) -> usize {
        match self {
            Components::F32(components) => components.len(),
            Components::U8(components) => components.len(),
            Components::U16(components) => components.len(),
            Components::U32(components) => components.len(),
            Components::Bool(components) => components.len(),
            Components::String(components) => components.len(),
        }
    }

    /// The component type.
    pub fn scalar(&self) -> Scalar {
        match self {
            Components::F32(_) => Scalar::F32,
            Components::U8(_) => Scalar::U8,
            Components::U16(_) => Scalar::U16,
            Components::U32(_) => Scalar::U32,
            Components::Bool(_) => Scalar::Bool,
            Components::String(_) => Scalar::String,
        }
    }
}
