//! Reads and writes `.gltf` and `.glb` bytes, gated behind the `codec`
//! feature.

mod from_gltf_bytes;
mod gltf_bytes;
mod gltf_loose_uris;
mod to_glb_bytes;
mod to_gltf_bytes;

pub use from_gltf_bytes::*;
pub use gltf_bytes::*;
pub use gltf_loose_uris::*;
pub use to_glb_bytes::*;
pub use to_gltf_bytes::*;

// Re-exported so a caller can refer to the dependencies the functions here
// take and bind them through the impl behind `impl`.
pub use crate::dependencies;
