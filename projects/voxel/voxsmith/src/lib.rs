#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! Every operation runs over voxcore's [`VoxMain`](voxcore::VoxMain) and never
//! sees a file format or the filesystem. The file formats live in voxconv,
//! meshconv, and sdfconv. `operations` holds a module per vxl command group,
//! each behind a feature of the same name. `utilities` holds what the
//! operations share.
//! The caller supplies the operations' dependencies through `dependencies`.
//! [`DependenciesImpl`](dependencies::DependenciesImpl), behind the `impl`
//! feature, binds them.

// Public API

pub mod dependencies;
pub mod operations;
pub mod utilities;

mod error;
mod result;

pub use error::*;
pub use result::*;

// Test support

#[cfg(test)]
mod test_utilities;
