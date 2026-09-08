#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between MagicaVoxel files and the voxcore state.
//!
//! [`from_mvox_file`] loads a decoded [`MVoxFile`](mvox::MVoxFile) into a
//! bare [`VoxMain`](voxcore::VoxMain). [`to_mvox_file`] writes one back as a
//! file synthesized from the scene. The `codec` module, behind the default
//! `codec` feature, goes straight to and from `.vox` bytes over mvox-codec.
//! The `ext` feature, on by default, opens the `ext` module. There the
//! MagicaVoxel state with no native voxcore home rides as the state's ext.
//! The `_with_ext` converters write a loaded file back exactly.

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

#[cfg(feature = "ext")]
pub mod ext;

#[cfg(not(feature = "ext"))]
mod ext;

// Internal API

mod internal;
pub(crate) use internal::*;
