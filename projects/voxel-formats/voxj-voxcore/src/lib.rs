#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Json documents and the voxcore state.
//!
//! [`from_voxj_file`] loads a [`voxj::VoxjFile`] into a
//! [`VoxMain`](voxcore::VoxMain), and [`to_voxj_file`] encodes one back, with
//! [`VoxjWriteOptions`] picking the block encodings, the ext block, and the
//! edit state. Each takes the caller's voxj dependencies:
//! [`DecodeBase64`](voxj::DecodeBase64) to load, [`EncodeBase64`](voxj::EncodeBase64)
//! and [`CostVoxjObject`](voxj::CostVoxjObject) to write.
//! `voxj::DependenciesImpl` supplies all three. The `codec` module, behind
//! the default `codec` feature, goes straight to and from `.voxj` / `.voxjz`
//! bytes and takes the codec's dependencies too.
//! `voxj_codec::DependenciesImpl` supplies those and voxj's. The document's
//! `ext` block loads into the state's ext through its
//! [`VoxExtBlockCodec`](voxcore::ext::VoxExtBlockCodec) impl and writes back
//! through [`VoxExt`](voxcore::ext::VoxExt). A [`VoxjVoxMain`] carries the
//! block verbatim, whichever format owns it.

// Public API

mod edit_state_mode;
mod error;
mod from_voxj_file;
mod result;
mod to_voxj_file;
mod voxj_vox_main;
mod voxj_write_options;

pub use edit_state_mode::*;
pub use error::*;
pub use from_voxj_file::*;
pub use result::*;
pub use to_voxj_file::*;
pub use voxj_vox_main::*;
pub use voxj_write_options::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
