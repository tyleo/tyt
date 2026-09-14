use crate::operations::mesh::Computation;

/// A binding the run computes into the environment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputedBinding {
    /// The bound name.
    pub name: String,

    /// What computes into the name.
    pub computation: Computation,
}
