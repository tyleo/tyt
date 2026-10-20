//! The dependencies the operations take from the caller, in a module per
//! operations module that has any. [`DependenciesImpl`], behind the `impl`
//! feature, binds them.

// Optional API

#[cfg(feature = "mesh_doc")]
pub mod mesh_doc;

#[cfg(feature = "object")]
pub mod object;

#[cfg(feature = "sdf_doc")]
pub mod sdf_doc;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
