//! glTF 2.0. The writer options and their values are re-exported from
//! `gltf-meshdoc` so a caller can reach them without depending on it.

mod from_gltf_error;
#[allow(clippy::module_inception)]
mod gltf;
mod gltf_container;
mod gltf_dependencies;
mod gltf_write_format;

pub use gltf::*;
pub use gltf_container::*;
pub use gltf_dependencies::*;
pub use gltf_write_format::*;

#[cfg(feature = "impl")]
mod gltf_dependencies_impl;

#[cfg(feature = "ext")]
mod ext;

pub use ::gltf_meshdoc::{GltfImageStorage, GltfWriteOptions};
