// Public API

#[allow(clippy::module_inception)]
mod palette_quantize;

pub use palette_quantize::*;

// Internal API

mod internal;
pub(crate) use internal::*;
