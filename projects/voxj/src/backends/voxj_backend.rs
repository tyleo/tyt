use std::fmt::Debug;

/// Selects the in-memory representation of a [`VoxjFile`](crate::VoxjFile)'s
/// objects, so one document hierarchy serves multiple forms.
pub trait VoxjBackend {
    /// The object representation.
    type Object: Clone + Debug + PartialEq;
}
