#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Qubicle files and the voxcore state.
//!
//! Each of the three Qubicle formats has a loader and a writer over its
//! decoded file: [`from_qb_file`] / [`to_qb_file`] for Qubicle Binary,
//! [`from_qbt_file`] / [`to_qbt_file`] for Qubicle Binary Tree, and
//! [`from_qbcl_file`] / [`to_qbcl_file`] for Qubicle Construction Library.
//! The Qubicle state with no native voxcore home rides in the format's ext
//! ([`QbExt`], [`QbtExt`], or [`QbclExt`]) on the
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
mod qb_ext;
mod qb_ext_matrix;
mod qb_vox_main;
mod qbcl_ext;
mod qbcl_ext_metadata;
mod qbcl_ext_node;
mod qbcl_ext_node_body;
mod qbcl_ext_thumbnail;
mod qbcl_vox_main;
mod qbt_ext;
mod qbt_ext_node;
mod qbt_vox_main;
mod result;
mod to_qb_file;
mod to_qbcl_file;
mod to_qbt_file;

pub use error::*;
pub use from_qb_file::*;
pub use from_qbcl_file::*;
pub use from_qbt_file::*;
pub use qb_ext::*;
pub use qb_ext_matrix::*;
pub use qb_vox_main::*;
pub use qbcl_ext::*;
pub use qbcl_ext_metadata::*;
pub use qbcl_ext_node::*;
pub use qbcl_ext_node_body::*;
pub use qbcl_ext_thumbnail::*;
pub use qbcl_vox_main::*;
pub use qbt_ext::*;
pub use qbt_ext_node::*;
pub use qbt_vox_main::*;
pub use result::*;
pub use to_qb_file::*;
pub use to_qbcl_file::*;
pub use to_qbt_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
