#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Json documents and a voxcore main.
//!
//! The main is a [`VoxjVoxMain`], a [`VoxMain`](voxcore::VoxMain) carrying
//! the document's `ext` block as a [`VoxjVoxExt`]. [`from_voxj_file`] loads
//! a [`voxj::VoxjFile`] into one and [`to_voxj_file`] encodes one back, with
//! [`VoxjWriteOptions`] picking the block encodings and the edit state.
//! [`to_voxj_vox_main`] gives a bare `VoxMain<()>` an empty block, and
//! `take_ext` takes the block back off. The block is not understood: it
//! follows no hook and goes stale under a mutation that moves a listing. A
//! crate that knows the block's entries takes it off and puts on an ext that
//! follows. Each converter takes the caller's voxj dependencies:
//! [`DecodeBase64`](voxj::DecodeBase64) to load,
//! [`EncodeBase64`](voxj::EncodeBase64) and
//! [`CostVoxjObject`](voxj::CostVoxjObject) to write.
//! `voxj::DependenciesImpl` supplies all three. The `codec` module, behind
//! the default `codec` feature, goes straight to and from `.voxj` / `.voxjz`
//! bytes and takes the codec's dependencies too.
//! `voxj_codec::DependenciesImpl` supplies those and voxj's.

// Public API

mod edit_state_mode;
mod error;
mod ext;
mod from_voxj_file;
mod result;
mod to_voxj_file;
mod to_voxj_vox_main;
mod voxj_vox_main;
mod voxj_write_options;

pub use edit_state_mode::*;
pub use error::*;
pub use ext::*;
pub use from_voxj_file::*;
pub use result::*;
pub use to_voxj_file::*;
pub use to_voxj_vox_main::*;
pub use voxj_vox_main::*;
pub use voxj_write_options::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
