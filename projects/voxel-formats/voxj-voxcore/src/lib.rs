#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Json documents and the voxcore state.
//!
//! [`from_voxj_file`] loads a [`voxj::VoxjFile`] into a
//! [`VoxMain`](voxcore::VoxMain), and [`to_voxj_file`] encodes one back, with
//! [`VoxjFileBuilder`] for control over the block encodings, the ext block,
//! and the edit state. Each takes the caller's voxj dependencies:
//! [`DecodeBase64`](voxj::DecodeBase64) to load, [`EncodeBase64`](voxj::EncodeBase64)
//! and [`CostVoxjObject`](voxj::CostVoxjObject) to write.
//! `voxj::DependenciesImpl` supplies all three. The [`codec`] module, behind
//! the default `codec` feature, goes straight to and from `.voxj` / `.voxjz`
//! bytes. The document's `ext` block goes through the state's ext slot, typed
//! by the slot's [`VoxExtSlot`](voxcore::ext::VoxExtSlot). A [`VoxjVoxMain`]
//! carries the block verbatim, whichever format owns it.

// Public API

mod edit_state_mode;
mod error;
mod from_voxj_file;
mod result;
mod to_voxj_file;
mod voxj_file_builder;
mod voxj_vox_main;

pub use edit_state_mode::*;
pub use error::*;
pub use from_voxj_file::*;
pub use result::*;
pub use to_voxj_file::*;
pub use voxj_file_builder::*;
pub use voxj_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
