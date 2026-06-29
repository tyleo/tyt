#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxMain`](voxcore::VoxMain) is the in-memory hub for every voxel
//! format this crate handles. For Voxel Json it converts both at the document
//! level ([`from_voxj_file`] / [`to_voxj_file`], with
//! [`VoxjFileBuilder`] for control over the block encoding, ext block, and edit
//! state) and straight to
//! and from `.voxj` / `.voxjz` bytes ([`from_voxj_bytes`],
//! [`to_voxj_bytes`], [`to_voxjz_bytes`]). For Voxel Max it
//! converts to and from the
//! lossless `VMaxFile` ([`from_vmax_file`] /
//! [`to_vmax_file`]), stashing the Voxel Max state with no
//! native home under a `voxel-max` ext. For MagicaVoxel it converts to and from a
//! decoded `MVoxFile` ([`from_mvox_file`] / [`to_mvox_file`])
//! and straight to and from `.vox` bytes ([`from_mvox_bytes`] /
//! [`to_mvox_bytes`]), stashing the MagicaVoxel state with no native
//! home under a `magica-voxel` ext. For Goxel it converts to and from a decoded
//! `GoxlFile` ([`from_goxl_file`] / [`to_goxl_file`]) and straight to and from
//! `.gox` bytes ([`from_goxl_bytes`] / [`to_goxl_bytes`]), stashing the Goxel
//! state with no native home under a `goxel` ext. For the three Qubicle formats
//! it converts to and from their decoded files ([`from_qb_file`] /
//! [`to_qb_file`], [`from_qbt_file`] / [`to_qbt_file`], [`from_qbcl_file`] /
//! [`to_qbcl_file`]) and straight to and from `.qb` / `.qbt` / `.qbcl` bytes (the
//! matching `from_*_bytes` / `to_*_bytes`), stashing the Qubicle state with no
//! native home under a `qubicle-qb`, `qubicle-qbt`, or `qubicle-qbcl` ext.

mod internal;
pub(crate) use internal::*;

mod edit_state_mode;
mod error;
mod from_goxl_bytes;
mod from_goxl_file;
mod from_mvox_bytes;
mod from_mvox_file;
mod from_qb_bytes;
mod from_qb_file;
mod from_qbcl_bytes;
mod from_qbcl_file;
mod from_qbt_bytes;
mod from_qbt_file;
mod from_vmax_file;
mod from_voxj_bytes;
mod from_voxj_file;
mod result;
mod scene_camera_source;
mod to_goxl_bytes;
mod to_goxl_file;
mod to_mvox_bytes;
mod to_mvox_file;
mod to_qb_bytes;
mod to_qb_file;
mod to_qbcl_bytes;
mod to_qbcl_file;
mod to_qbt_bytes;
mod to_qbt_file;
mod to_vmax_file;
mod to_voxj_bytes;
mod to_voxj_bytes_with;
mod to_voxj_file;
mod to_voxjz_bytes;
mod to_voxjz_bytes_with;
mod vmax_file_builder;
mod voxel_max_color_format;
mod voxj_file_builder;

pub use edit_state_mode::*;
pub use error::*;
pub use from_goxl_bytes::*;
pub use from_goxl_file::*;
pub use from_mvox_bytes::*;
pub use from_mvox_file::*;
pub use from_qb_bytes::*;
pub use from_qb_file::*;
pub use from_qbcl_bytes::*;
pub use from_qbcl_file::*;
pub use from_qbt_bytes::*;
pub use from_qbt_file::*;
pub use from_vmax_file::*;
pub use from_voxj_bytes::*;
pub use from_voxj_file::*;
pub use result::*;
pub use scene_camera_source::*;
pub use to_goxl_bytes::*;
pub use to_goxl_file::*;
pub use to_mvox_bytes::*;
pub use to_mvox_file::*;
pub use to_qb_bytes::*;
pub use to_qb_file::*;
pub use to_qbcl_bytes::*;
pub use to_qbcl_file::*;
pub use to_qbt_bytes::*;
pub use to_qbt_file::*;
pub use to_vmax_file::*;
pub use to_voxj_bytes::*;
pub use to_voxj_bytes_with::*;
pub use to_voxj_file::*;
pub use to_voxjz_bytes::*;
pub use to_voxjz_bytes_with::*;
pub use vmax_file_builder::*;
pub use voxel_max_color_format::*;
pub use voxj_file_builder::*;

// Re-exported so callers can name the camera passed to `SceneCameraSource::Camera`.
pub use vmax::VMaxSceneCamera;

pub(crate) use to_vmax_file::write_vmax;
