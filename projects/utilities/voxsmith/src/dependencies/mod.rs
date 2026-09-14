//! The dependencies the operations take, a module per operation that has
//! any. [`DependenciesImpl`], behind the `impl` feature, binds them over
//! `png` and `zune-jpeg`.

#[cfg(feature = "mesh")]
pub mod mesh;

#[cfg(feature = "voxelize")]
pub mod voxelize;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
