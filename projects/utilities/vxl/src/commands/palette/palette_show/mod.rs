// Public API

#[allow(clippy::module_inception)]
mod palette_show;

pub use palette_show::*;

// Internal API

mod internal;
pub(crate) use internal::*;
