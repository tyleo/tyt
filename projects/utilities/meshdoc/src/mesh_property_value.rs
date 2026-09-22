use crate::{BMeshFile, MeshTextureRef};
use branded_id::U32Id;

/// The value of a [`MeshProperty`](crate::MeshProperty). A list or rows
/// variant holds one entry per element of whatever the owner counts over,
/// such as a row per palette entry.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshPropertyValue {
    /// A boolean.
    Bool(bool),

    /// A list of booleans.
    Bools(Vec<bool>),

    /// A list of boolean rows, such as a mask per entry.
    BoolRows(Vec<Vec<bool>>),

    /// An integer.
    Int(i64),

    /// A list of integers.
    Ints(Vec<i64>),

    /// A list of integer rows.
    IntRows(Vec<Vec<i64>>),

    /// A finite float.
    Float(f64),

    /// A list of finite floats, such as a vector or a color.
    Floats(Vec<f64>),

    /// A list of float rows, such as a vector per entry.
    FloatRows(Vec<Vec<f64>>),

    /// A string.
    Text(String),

    /// A list of strings.
    Texts(Vec<String>),

    /// A list of string rows.
    TextRows(Vec<Vec<String>>),

    /// A texture reference.
    Texture(MeshTextureRef),

    /// One of the document's files.
    File(U32Id<BMeshFile>),
}

impl MeshPropertyValue {
    /// Whether every float the value holds is finite.
    pub fn is_finite(&self) -> bool {
        match self {
            MeshPropertyValue::Float(value) => value.is_finite(),
            MeshPropertyValue::Floats(values) => values.iter().all(|value| value.is_finite()),
            MeshPropertyValue::FloatRows(rows) => rows
                .iter()
                .all(|row| row.iter().all(|value| value.is_finite())),
            _ => true,
        }
    }

    /// The texture reference, when the value is one.
    pub fn texture_ref(&self) -> Option<MeshTextureRef> {
        match self {
            MeshPropertyValue::Texture(texture_ref) => Some(*texture_ref),
            _ => None,
        }
    }

    /// The file id, when the value is one.
    pub fn file_id(&self) -> Option<U32Id<BMeshFile>> {
        match self {
            MeshPropertyValue::File(file_id) => Some(*file_id),
            _ => None,
        }
    }
}
