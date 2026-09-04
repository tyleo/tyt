#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Qubicle files and the voxcore state.
//!
//! Each of the three Qubicle formats has a loader and a writer over its
//! decoded file: [`from_qb_file`] / [`to_qb_file`] for Qubicle Binary,
//! [`from_qbt_file`] / [`to_qbt_file`] for Qubicle Binary Tree, and
//! [`from_qbcl_file`] / [`to_qbcl_file`] for Qubicle Construction Library.
//! The Qubicle state with no native voxcore home rides in the format's ext
//! ([`QubicleQbExt`], [`QubicleQbtExt`], or [`QubicleQbclExt`]) on the
//! state's ext slot, so a loaded file writes back exactly. The `.qb` and
//! `.qbt` writers require it. A state without one, such as one loaded from
//! another format, has its `.qbcl` file synthesized from the bare scene. The
//! [`codec`] module, behind the
//! default `codec` feature, goes straight to and from file bytes. Its `.qbt`
//! and `.qbcl` conversions take the codec's dependencies, which
//! `qbcl_codec::DependenciesImpl` supplies. The `ext` feature keys each ext
//! into a document's `ext` block through voxcore's
//! [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_qb_file;
mod from_qbcl_file;
mod from_qbt_file;
mod qubicle_qb_ext;
mod qubicle_qb_matrix;
mod qubicle_qb_vox_main;
mod qubicle_qbcl_ext;
mod qubicle_qbcl_metadata;
mod qubicle_qbcl_node;
mod qubicle_qbcl_node_body;
mod qubicle_qbcl_thumbnail;
mod qubicle_qbcl_vox_main;
mod qubicle_qbt_ext;
mod qubicle_qbt_node;
mod qubicle_qbt_vox_main;
mod result;
mod to_qb_file;
mod to_qbcl_file;
mod to_qbt_file;

pub use error::*;
pub use from_qb_file::*;
pub use from_qbcl_file::*;
pub use from_qbt_file::*;
pub use qubicle_qb_ext::*;
pub use qubicle_qb_matrix::*;
pub use qubicle_qb_vox_main::*;
pub use qubicle_qbcl_ext::*;
pub use qubicle_qbcl_metadata::*;
pub use qubicle_qbcl_node::*;
pub use qubicle_qbcl_node_body::*;
pub use qubicle_qbcl_thumbnail::*;
pub use qubicle_qbcl_vox_main::*;
pub use qubicle_qbt_ext::*;
pub use qubicle_qbt_node::*;
pub use qubicle_qbt_vox_main::*;
pub use result::*;
pub use to_qb_file::*;
pub use to_qbcl_file::*;
pub use to_qbt_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
