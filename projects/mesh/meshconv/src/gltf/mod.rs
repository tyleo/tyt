//! glTF 2.0. The writer options and their values are re-exported from
//! `gltf-meshdoc` so a caller can reach them without depending on it.

// Public API

#[allow(clippy::module_inception)]
mod gltf;
mod gltf_container;
mod gltf_dependencies;
mod gltf_write_format;

pub use gltf::*;
pub use gltf_container::*;
pub use gltf_dependencies::*;
pub use gltf_write_format::*;

pub use ::gltf_meshdoc::{GltfImageStorage, GltfWriteOptions};

// Optional API

#[cfg(feature = "ext")]
mod gltf_format_ext;

#[cfg(feature = "impl")]
mod gltf_dependencies_impl;

// Internal API

mod document_files;
mod from_gltf_error;
mod loose_files;

pub(crate) use document_files::*;
pub(crate) use loose_files::*;
