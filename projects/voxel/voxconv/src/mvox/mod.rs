//! MagicaVoxel: the [`MVox`] format.

// Public API

#[allow(clippy::module_inception)]
mod mvox;

pub use mvox::*;

// Optional API

#[cfg(feature = "ext")]
mod mvox_format_ext;

// Internal API

mod from_mvox_error;
