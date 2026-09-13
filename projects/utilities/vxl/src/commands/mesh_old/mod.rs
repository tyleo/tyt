// Public API

#[allow(clippy::module_inception)]
mod mesh_old;

pub use mesh_old::*;

// Internal API

mod internal;
pub(crate) use internal::*;
