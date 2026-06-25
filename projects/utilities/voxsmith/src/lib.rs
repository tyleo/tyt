#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxState`](voxcore::VoxState) is the in-memory hub for every voxel
//! format this crate handles. For Voxel Json it converts both at the document
//! level ([`from_voxj_file`] / [`to_voxj_file`], with
//! [`to_voxj_file_with`] for a fixed block encoding) and straight to
//! and from `.voxj` / `.voxjz` bytes ([`from_voxj_bytes`],
//! [`to_voxj_bytes`], [`to_voxjz_bytes`]). For Voxel Max it
//! converts to and from the
//! lossless `VMaxFile` ([`from_vmax_file`] /
//! [`to_vmax_file`]), stashing the Voxel Max state with no
//! native home under a `voxel-max` ext. For MagicaVoxel it converts to and from a
//! decoded `MVoxFile` ([`from_mvox_file`] / [`to_mvox_file`])
//! and straight to and from `.vox` bytes ([`from_mvox_bytes`] /
//! [`to_mvox_bytes`]), stashing the MagicaVoxel state with no native
//! home under a `magica-voxel` ext.

mod internal;
pub(crate) use internal::*;

mod error;
mod from_mvox_bytes;
mod from_mvox_file;
mod from_vmax_file;
mod from_voxj_bytes;
mod from_voxj_file;
mod result;
mod to_mvox_bytes;
mod to_mvox_file;
mod to_vmax_file;
mod to_voxj_bytes;
mod to_voxj_bytes_with;
mod to_voxj_file;
mod to_voxj_file_with;
mod to_voxjz_bytes;
mod to_voxjz_bytes_with;
mod voxel_max_color_format;

pub use error::*;
pub use from_mvox_bytes::*;
pub use from_mvox_file::*;
pub use from_vmax_file::*;
pub use from_voxj_bytes::*;
pub use from_voxj_file::*;
pub use result::*;
pub use to_mvox_bytes::*;
pub use to_mvox_file::*;
pub use to_vmax_file::*;
pub use to_voxj_bytes::*;
pub use to_voxj_bytes_with::*;
pub use to_voxj_file::*;
pub use to_voxj_file_with::*;
pub use to_voxjz_bytes::*;
pub use to_voxjz_bytes_with::*;
pub use voxel_max_color_format::*;
