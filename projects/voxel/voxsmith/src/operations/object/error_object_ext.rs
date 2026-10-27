use crate::{Error, operations::object::MeshElement};
use std::fmt::Display;

impl Error {
    /// Builds an [`Error::MeshRecord`] from the element and its reason.
    pub(crate) fn mesh_record(element: MeshElement, reason: impl Display) -> Self {
        Error::MeshRecord {
            element,
            reason: reason.to_string(),
        }
    }
}
