#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between MagicaVoxel files and the voxcore state.
//!
//! [`from_mvox_file`] loads a decoded [`MVoxFile`](mvox::MVoxFile) into a bare
//! [`VoxMain`](voxcore::VoxMain). [`to_mvox_file`] writes one back as a file
//! synthesized from the scene. The `codec` module, behind the default `codec`
//! feature, goes straight to and from `.vox` bytes over mvox-codec. The `ext`
//! module carries the MagicaVoxel state with no native voxcore home as the
//! state's ext. The `_with_ext` converters write a loaded file back exactly.
//! The ext follows the listings through the [`VoxExt`](voxcore::VoxExt) hooks,
//! so a state mutated after the load still writes back with the surviving
//! provenance. The `serde` feature, on by default, derives serde for the ext
//! types.

// Public API

mod error;
mod from_mvox_file;
mod result;
mod to_mvox_file;

pub use error::*;
pub use from_mvox_file::*;
pub use result::*;
pub use to_mvox_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

pub mod ext;

// Internal API

mod internal;
pub(crate) use internal::*;
