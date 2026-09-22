#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between glTF files and the meshdoc state.
//!
//! The state is a [`GltfMeshMain`], a [`MeshMain`](meshdoc::MeshMain) carrying
//! a [`GltfExt`] with the glTF state that has no native meshdoc home.
//! [`from_gltf_file`] loads a [`GltfFile`] into one. [`to_gltf_file`] writes
//! one back, exactly for a loaded file, with [`GltfWriteOptions`] picking where
//! the images go. [`to_gltf_mesh_main`] gives a bare `MeshMain<()>` a
//! synthesized ext, and `take_ext` takes the ext back off. The ext keys an
//! entry per entity by id and follows the state through meshdoc's
//! [`MeshExt`](meshdoc::MeshExt) hooks, so a state mutated after the load still
//! writes back with a complete ext. Geometry and node transforms copy straight
//! because meshdoc shares glTF's frame. The `codec` module, behind the default
//! `codec` feature, goes straight to and from `.gltf` and `.glb` bytes. The
//! data URIs of both go through the caller's [`DecodeBase64`] and
//! [`EncodeBase64`], which `DependenciesImpl` supplies behind the `impl`
//! feature. The `serde` feature, on by default, derives serde for the ext
//! types.

// Public API

pub mod dependencies;

mod error;
mod ext;
mod from_gltf_file;
mod gltf_file;
mod gltf_image_storage;
mod gltf_mesh_main;
mod gltf_write_options;
mod result;
mod to_gltf_file;
mod to_gltf_mesh_main;

pub use dependencies::*;
pub use error::*;
pub use ext::*;
pub use from_gltf_file::*;
pub use gltf_file::*;
pub use gltf_image_storage::*;
pub use gltf_mesh_main::*;
pub use gltf_write_options::*;
pub use result::*;
pub use to_gltf_file::*;
pub use to_gltf_mesh_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;

// Test support.

#[cfg(all(test, feature = "impl"))]
mod test;

#[cfg(all(test, feature = "impl"))]
pub(crate) use test::*;
