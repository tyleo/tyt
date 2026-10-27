//! Goxel: the [`Goxl`] format and the dependencies its codec takes.

// Public API

#[allow(clippy::module_inception)]
mod goxl;
mod goxl_dependencies;

pub use goxl::*;
pub use goxl_dependencies::*;

// Optional API

#[cfg(feature = "ext")]
mod goxl_format_ext;

#[cfg(feature = "impl")]
mod goxl_dependencies_impl;

// Internal API

mod from_goxl_error;
