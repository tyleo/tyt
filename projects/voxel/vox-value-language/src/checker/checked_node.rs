use crate::{CheckedKind, Type};

/// A node of the checked tree, its type settled.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckedNode {
    /// What the node computes.
    pub(crate) kind: CheckedKind,

    /// The type the node computes.
    pub(crate) output: Type,
}
