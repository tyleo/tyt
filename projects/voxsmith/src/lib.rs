#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxState`](voxcore::VoxState) is the in-memory representation of
//! a voxj document. This crate converts between the two, both at the value level
//! ([`vox_state_from_voxj_codec_main`] / [`voxj_codec_main_from_vox_state`]) and
//! straight to and from `.voxj` / `.voxjz` bytes
//! ([`vox_state_from_voxj_bytes`], [`vox_state_to_voxj_bytes`],
//! [`vox_state_to_voxjz_bytes`]).

mod internal;
pub(crate) use internal::*;

mod vox_state_from_voxj_bytes;
mod vox_state_from_voxj_codec_main;
mod vox_state_to_voxj_bytes;
mod vox_state_to_voxjz_bytes;
mod voxj_codec_main_from_vox_state;

pub use vox_state_from_voxj_bytes::*;
pub use vox_state_from_voxj_codec_main::*;
pub use vox_state_to_voxj_bytes::*;
pub use vox_state_to_voxjz_bytes::*;
pub use voxj_codec_main_from_vox_state::*;
