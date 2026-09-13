/// The components of a [`MeshVertexAttribute`](crate::MeshVertexAttribute),
/// flattened vertex by vertex in the scalar type the attribute stores.
#[derive(Clone, Debug, PartialEq)]
pub enum MeshAttributeComponents {
    /// Finite floats.
    F64(Vec<f64>),

    /// Unsigned 8-bit integers.
    U8(Vec<u8>),

    /// Unsigned 16-bit integers.
    U16(Vec<u16>),
}

impl MeshAttributeComponents {
    /// The component count.
    pub fn len(&self) -> usize {
        match self {
            MeshAttributeComponents::F64(components) => components.len(),
            MeshAttributeComponents::U8(components) => components.len(),
            MeshAttributeComponents::U16(components) => components.len(),
        }
    }

    /// Whether there are no components.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The index of the first non-finite component, or `None` when every
    /// component is finite. An integer component is always finite.
    pub fn first_non_finite_index(&self) -> Option<usize> {
        match self {
            MeshAttributeComponents::F64(components) => components
                .iter()
                .position(|component| !component.is_finite()),
            _ => None,
        }
    }
}
