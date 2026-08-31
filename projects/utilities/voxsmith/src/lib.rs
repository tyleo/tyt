#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxMain`](voxcore::VoxMain) is the in-memory hub for every voxel
//! format this crate handles. Each format's loader returns its `*VoxMain`
//! alias, whose ext slot holds the format's ext, the state with no native
//! voxcore home, so the format's writer can rebuild the file exactly. A state
//! converted across formats drops that ext explicitly through
//! [`VoxMain::map_ext`](voxcore::VoxMain::map_ext). For Voxel Json,
//! voxj-voxcore converts the document and carries its `ext` block verbatim;
//! here [`from_voxj_file`] and [`from_voxj_bytes`] type that block into the
//! slot, each format's ext keyed into the block by its [`VoxjExtCodec`] and
//! a whole slot handled through its [`VoxjExtSlot`]. For Voxel Max it
//! converts to and from the lossless `VMaxFile`
//! ([`from_vmax_file`] / [`to_vmax_file`]), carrying the [`VoxelMaxExt`]. For
//! MagicaVoxel it converts to and from a decoded `MVoxFile`
//! ([`from_mvox_file`] / [`to_mvox_file`]) and straight to and from `.vox`
//! bytes ([`from_mvox_bytes`] / [`to_mvox_bytes`]), carrying the
//! [`MagicaVoxelExt`]. For Goxel it converts to and from a decoded `GoxlFile`
//! ([`from_goxl_file`] / [`to_goxl_file`]) and straight to and from `.gox`
//! bytes ([`from_goxl_bytes`] / [`to_goxl_bytes`]), carrying the
//! [`GoxelExt`]. For the three Qubicle formats it converts to and from their
//! decoded files ([`from_qb_file`] / [`to_qb_file`], [`from_qbt_file`] /
//! [`to_qbt_file`], [`from_qbcl_file`] / [`to_qbcl_file`]) and straight to and
//! from `.qb` / `.qbt` / `.qbcl` bytes (the matching `from_*_bytes` /
//! `to_*_bytes`), carrying the [`QubicleQbExt`], [`QubicleQbtExt`], or
//! [`QubicleQbclExt`].

#[cfg(not(feature = "_codec"))]
compile_error!(
    "voxsmith needs at least one codec feature enabled: goxl, mvox, qbcl, vmax, or voxj"
);

#[cfg(any(feature = "_codec", feature = "_mesh"))]
mod internal;
#[cfg(any(feature = "_codec", feature = "_mesh"))]
pub(crate) use internal::*;

#[cfg(feature = "_mesh")]
mod check_gltf_property_ranges;
mod color_space;
#[cfg(any(feature = "_codec", feature = "_mesh"))]
mod convert;
mod dither;
mod error;
mod gltf_properties;
mod gltf_property_kind;
#[cfg(feature = "_mesh")]
mod gltf_range;
mod order_palette_colors;
mod reduce_palette;
mod reduction_method;
mod result;

#[cfg(feature = "_mesh")]
pub use check_gltf_property_ranges::*;
pub use color_space::*;
#[cfg(any(feature = "_codec", feature = "_mesh"))]
pub use convert::*;
pub use dither::*;
pub use error::*;
pub use gltf_properties::*;
pub use gltf_property_kind::*;
#[cfg(feature = "_mesh")]
pub(crate) use gltf_range::*;
pub use order_palette_colors::*;
pub use reduce_palette::*;
pub use reduction_method::*;
pub use result::*;
