use crate::{Error, operations::object::RenderElement};
use std::fmt::Display;

impl Error {
    /// Builds an [`Error::RenderRecord`] from the element and its reason.
    pub(crate) fn render_record(element: RenderElement, reason: impl Display) -> Self {
        Error::RenderRecord {
            element,
            reason: reason.to_string(),
        }
    }
}
