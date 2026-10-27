// Public API

#[allow(clippy::module_inception)]
mod object_mesh;

pub use object_mesh::*;

// Internal API

mod internal;
pub(crate) use internal::*;
