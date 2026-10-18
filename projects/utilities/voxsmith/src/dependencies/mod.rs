//! The dependencies the operations take from the caller, in a module per
//! operations module that has any. [`DependenciesImpl`], behind the `impl`
//! feature, binds them over `png`, `zune-jpeg`, and `voxrender-cpu`.

// Optional API

#[cfg(feature = "mesh_doc")]
pub mod mesh_doc;

#[cfg(feature = "object")]
pub mod object;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
