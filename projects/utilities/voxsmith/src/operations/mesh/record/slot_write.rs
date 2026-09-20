use crate::operations::mesh::SlotSource;

/// A material slot write; the property names a modeled material field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SlotWrite {
    /// The destination property.
    pub property: String,

    /// What fills the property.
    pub source: SlotSource,
}
