#![deny(rustdoc::broken_intra_doc_links)]

//! The TypeScript builder that records a voxel model as an SDF Json (`.sdfj`)
//! document, embedded for a Rust host to run. A run writes
//! [`SDFJ_BUILDER_FILES`], the libraries at [`SDFJ_BUILDER_LIBRARIES_PATH`], and
//! the build variables at [`SDFJ_BUILDER_VARS_PATH`] under one directory and
//! then starts a [`JavaScriptRuntime`] on the builder.

// Public API

mod javascript_runtime;
mod sdfj_builder_file;
mod sdfj_builder_files;
mod sdfj_builder_libraries_path;
mod sdfj_builder_vars_path;

pub use javascript_runtime::*;
pub use sdfj_builder_file::*;
pub use sdfj_builder_files::*;
pub use sdfj_builder_libraries_path::*;
pub use sdfj_builder_vars_path::*;
