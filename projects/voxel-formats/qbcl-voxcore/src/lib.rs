#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Qubicle files and the voxcore state.
//!
//! Each of the three Qubicle formats has a loader and a writer over its
//! decoded file: [`from_qb_file`] / [`to_qb_file`] for Qubicle Binary,
//! [`from_qbt_file`] / [`to_qbt_file`] for Qubicle Binary Tree, and
//! [`from_qbcl_file`] / [`to_qbcl_file`] for Qubicle Construction Library.
//! A loader returns a bare [`VoxMain`](voxcore::VoxMain). A writer
//! synthesizes the file from the scene. The `codec` module, behind the
//! default `codec` feature, goes straight to and from file bytes. Its `.qbt`
//! and `.qbcl` conversions take the codec's dependencies, which
//! `qbcl_codec::DependenciesImpl` supplies. The `ext` feature, on by default,
//! opens the `ext` module. There the Qubicle state with no native voxcore
//! home rides as the state's ext. The `_with_ext` converters write a loaded
//! file back exactly. The `qb` and `qbt` exts follow the listings through the
//! [`VoxExt`](voxcore::ext::VoxExt) hooks, so a state mutated after the load
//! still writes back with the surviving provenance.

// Public API

mod error;
mod from_qb_file;
mod from_qbcl_file;
mod from_qbt_file;
mod result;
mod to_qb_file;
mod to_qbcl_file;
mod to_qbt_file;

pub use error::*;
pub use from_qb_file::*;
pub use from_qbcl_file::*;
pub use from_qbt_file::*;
pub use result::*;
pub use to_qb_file::*;
pub use to_qbcl_file::*;
pub use to_qbt_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
pub mod ext;

#[cfg(not(feature = "ext"))]
mod ext;

// Internal API

mod internal;
pub(crate) use internal::*;
