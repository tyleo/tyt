#![deny(rustdoc::broken_intra_doc_links)]

//! Utilities for working with voxels.
//!
//! voxcore's [`VoxMain`](voxcore::VoxMain) is the in-memory hub for every voxel
//! format this crate handles. Each format's loader returns its `*VoxMain`
//! alias, whose ext slot holds the format's ext, the state with no native
//! voxcore home, so the format's writer can rebuild the file exactly. A state
//! converted across formats drops that ext explicitly through
//! [`VoxMain::map_ext`](voxcore::VoxMain::map_ext). The `ext` feature keys
//! each format's ext into a document's `ext` block through voxcore's
//! [`VoxExtCodec`](voxcore::ext::VoxExtCodec). For Voxel Json the crate
//! fronts voxj-voxcore's typed loads and writes, which carry that block
//! across formats ([`from_voxj_bytes`] / [`to_voxj_bytes`], with
//! [`VoxjFileBuilder`] over the block encodings, the ext block, and the
//! edit state), and runs the spec checks over a document
//! ([`check_voxj_bytes`]). For Voxel Max it fronts vmax-voxcore's typed
//! loads and writes, to and from the lossless `VMaxFile` ([`from_vmax_file`]
//! / [`to_vmax_file`]) and straight to and from a package's files
//! ([`from_vmax_package`] / [`to_vmax_package`], with [`VmaxFileBuilder`]
//! over the color format and the scene camera), carrying the
//! [`VoxelMaxExt`]. For MagicaVoxel it converts to and from a decoded
//! `MVoxFile`
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

// Re-exported so a consumer names the state types every converter here
// exchanges through this crate.
pub use voxcore;

#[cfg(any(feature = "_color", feature = "_mesh", feature = "report"))]
mod internal;
#[cfg(any(feature = "_color", feature = "_mesh", feature = "report"))]
pub(crate) use internal::*;

#[cfg(feature = "_mesh")]
mod check_gltf_property_ranges;
#[cfg(feature = "_mesh")]
mod color_range;
mod color_space;
#[cfg(any(feature = "_codec", feature = "_mesh"))]
mod convert;
#[cfg(feature = "gltf")]
mod default_lin_srgba_f64_color;
mod dither;
mod error;
#[cfg(feature = "_mesh")]
mod gltf_range;
mod index_range;
mod order_palette_colors;
mod palette_reduction;
mod reduce_palette;
mod reduction_method;
#[cfg(feature = "report")]
mod report;
mod result;
#[cfg(feature = "_mesh")]
mod scalar_range;
#[cfg(feature = "select")]
mod select;
mod vector_component;
mod voxel_format;

#[cfg(feature = "_mesh")]
pub use check_gltf_property_ranges::*;
#[cfg(feature = "_mesh")]
pub(crate) use color_range::*;
pub use color_space::*;
#[cfg(any(feature = "_codec", feature = "_mesh"))]
pub use convert::*;
#[cfg(feature = "gltf")]
pub(crate) use default_lin_srgba_f64_color::*;
pub use dither::*;
pub use error::*;
#[cfg(feature = "_mesh")]
pub(crate) use gltf_range::*;
pub use index_range::*;
pub use order_palette_colors::*;
pub use palette_reduction::*;
pub use reduce_palette::*;
pub use reduction_method::*;
#[cfg(feature = "report")]
pub use report::*;
pub use result::*;
#[cfg(feature = "_mesh")]
pub(crate) use scalar_range::*;
#[cfg(feature = "select")]
pub use select::*;
pub use vector_component::*;
pub use voxel_format::*;
