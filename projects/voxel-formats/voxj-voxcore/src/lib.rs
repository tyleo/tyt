#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Json documents and the voxcore state.
//!
//! [`from_voxj_file`] loads a [`voxj::VoxjFile`] into a [`VoxjVoxMain`],
//! and [`to_voxj_file`] encodes one back, with [`VoxjFileBuilder`] for
//! control over the block encodings, ext block, and edit state. The [`codec`]
//! module, behind the default `codec` feature, goes straight to and from
//! `.voxj` / `.voxjz` bytes. The document's `ext` block is carried as a
//! voxcore value tree, whichever format owns it; a caller that types the
//! block maps it in and out of the slot itself.

#[cfg(feature = "codec")]
pub mod codec;

mod internal;
pub(crate) use internal::*;

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
