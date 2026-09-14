use crate::operations::mesh::{ArrayDomain, ExtraWrite, SlotWrite};

/// One material's elements.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct MaterialRecord {
    /// The material's name.
    pub name: Option<String>,

    /// The declared stream list; when absent, the list derives from use.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The slot writes.
    pub slots: Vec<SlotWrite>,

    /// The named properties.
    pub extras: Vec<ExtraWrite>,
}
