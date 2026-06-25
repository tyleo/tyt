#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxState`](voxcore::VoxState) is the in-memory hub for every voxel
//! format this crate handles. For Voxel Json it converts both at the value level
//! ([`vox_state_from_voxj_codec_main`] / [`voxj_codec_main_from_vox_state`]) and
//! straight to and from `.voxj` / `.voxjz` bytes
//! ([`vox_state_from_voxj_bytes`], [`vox_state_to_voxj_bytes`],
//! [`vox_state_to_voxjz_bytes`]). For Voxel Max it converts to and from a fully
//! decoded `VMaxCodecFile` ([`vox_state_from_vmax_codec_file`] /
//! [`vmax_codec_file_from_vox_state`]), stashing the Voxel Max state with no
//! native home under a `voxel-max` ext. For MagicaVoxel it converts to and from a
//! decoded `MVoxFile` ([`vox_state_from_mvox_file`] / [`mvox_file_from_vox_state`])
//! and straight to and from `.vox` bytes ([`vox_state_from_mvox_bytes`] /
//! [`mvox_bytes_from_vox_state`]), stashing the MagicaVoxel state with no native
//! home under a `magica-voxel` ext.

mod internal;
pub(crate) use internal::*;

mod error;
mod mvox_bytes_from_vox_state;
mod mvox_file_from_vox_state;
mod result;
mod vmax_codec_file_from_vox_state;
mod vox_state_from_mvox_bytes;
mod vox_state_from_mvox_file;
mod vox_state_from_vmax_codec_file;
mod vox_state_from_voxj_bytes;
mod vox_state_from_voxj_codec_main;
mod vox_state_to_voxj_bytes;
mod vox_state_to_voxjz_bytes;
mod voxel_max_color_format;
mod voxj_codec_main_from_vox_state;

pub use error::*;
pub use mvox_bytes_from_vox_state::*;
pub use mvox_file_from_vox_state::*;
pub use result::*;
pub use vmax_codec_file_from_vox_state::*;
pub use vox_state_from_mvox_bytes::*;
pub use vox_state_from_mvox_file::*;
pub use vox_state_from_vmax_codec_file::*;
pub use vox_state_from_voxj_bytes::*;
pub use vox_state_from_voxj_codec_main::*;
pub use vox_state_to_voxj_bytes::*;
pub use vox_state_to_voxjz_bytes::*;
pub use voxel_max_color_format::*;
pub use voxj_codec_main_from_vox_state::*;
