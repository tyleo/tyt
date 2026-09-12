#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Qubicle files and the voxcore state.
//!
//! Each Qubicle format has a [`VoxMain`](voxcore::VoxMain) carrying the
//! format's state with no native voxcore home as its ext: [`QbVoxMain`] for
//! Qubicle Binary, [`QbtVoxMain`] for Qubicle Binary Tree, and
//! [`QbclVoxMain`] for Qubicle Construction Library. [`from_qb_file`] loads
//! a decoded file into one and [`to_qb_file`] writes one back, exactly for a
//! loaded file. The `qbt` and `qbcl` pairs work the same way.
//! [`to_qb_vox_main`], [`to_qbt_vox_main`], and [`to_qbcl_vox_main`] give a
//! bare `VoxMain<()>` a synthesized ext, and `take_ext` takes the ext back
//! off. The exts follow the listings through
//! the [`VoxExt`](voxcore::VoxExt) hooks, so a state mutated after the load
//! still writes back with the surviving provenance. The `codec` module,
//! behind the default `codec` feature, goes straight to and from file bytes.
//! Its `.qbt` and `.qbcl` conversions take the codec's dependencies, which
//! `qbcl_codec::DependenciesImpl` supplies. The `serde` feature, on by
//! default, derives serde for the ext types.

// Public API

mod error;
mod ext;
mod from_qb_file;
mod from_qbcl_file;
mod from_qbt_file;
mod qb_vox_main;
mod qbcl_vox_main;
mod qbt_vox_main;
mod result;
mod to_qb_file;
mod to_qb_vox_main;
mod to_qbcl_file;
mod to_qbcl_vox_main;
mod to_qbt_file;
mod to_qbt_vox_main;

pub use error::*;
pub use ext::*;
pub use from_qb_file::*;
pub use from_qbcl_file::*;
pub use from_qbt_file::*;
pub use qb_vox_main::*;
pub use qbcl_vox_main::*;
pub use qbt_vox_main::*;
pub use result::*;
pub use to_qb_file::*;
pub use to_qb_vox_main::*;
pub use to_qbcl_file::*;
pub use to_qbcl_vox_main::*;
pub use to_qbt_file::*;
pub use to_qbt_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
