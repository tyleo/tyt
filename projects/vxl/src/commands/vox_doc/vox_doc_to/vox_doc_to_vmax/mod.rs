// Public API

#[allow(clippy::module_inception)]
mod vox_doc_to_vmax;

pub use vox_doc_to_vmax::*;

// Internal API

mod internal;
pub(crate) use internal::*;
