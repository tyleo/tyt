#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxMain`](voxcore::VoxMain) is the in-memory hub for every voxel
//! format this crate handles. For Voxel Json it converts both at the document
//! level ([`from_voxj_file`] / [`to_voxj_file`], with [`VoxjFileBuilder`] for
//! control over the block encoding, ext block, and edit state) and straight to
//! and from `.voxj` / `.voxjz` bytes ([`from_voxj_bytes`], [`to_voxj_bytes`],
//! [`to_voxjz_bytes`]). For Voxel Max it converts to and from the lossless
//! `VMaxFile` ([`from_vmax_file`] / [`to_vmax_file`]), stashing the Voxel Max
//! state with no native home under a `voxel-max` ext. For MagicaVoxel it
//! converts to and from a decoded `MVoxFile` ([`from_mvox_file`] /
//! [`to_mvox_file`]) and straight to and from `.vox` bytes ([`from_mvox_bytes`]
//! / [`to_mvox_bytes`]), stashing the MagicaVoxel state with no native home
//! under a `magica-voxel` ext. For Goxel it converts to and from a decoded
//! `GoxlFile` ([`from_goxl_file`] / [`to_goxl_file`]) and straight to and from
//! `.gox` bytes ([`from_goxl_bytes`] / [`to_goxl_bytes`]), stashing the Goxel
//! state with no native home under a `goxel` ext. For the three Qubicle formats
//! it converts to and from their decoded files ([`from_qb_file`] /
//! [`to_qb_file`], [`from_qbt_file`] / [`to_qbt_file`], [`from_qbcl_file`] /
//! [`to_qbcl_file`]) and straight to and from `.qb` / `.qbt` / `.qbcl` bytes
//! (the matching `from_*_bytes` / `to_*_bytes`), stashing the Qubicle state
//! with no native home under a `qubicle-qb`, `qubicle-qbt`, or `qubicle-qbcl`
//! ext.

mod internal;
pub(crate) use internal::*;

mod convert;
mod error;
mod result;

pub use convert::*;
pub use error::*;
pub use result::*;
