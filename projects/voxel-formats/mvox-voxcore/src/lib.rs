#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between MagicaVoxel files and the voxcore state.
//!
//! The state is a [`MVoxVoxMain`], a [`VoxMain`](voxcore::VoxMain) carrying a
//! [`MVoxExt`] with the MagicaVoxel state that has no native voxcore home.
//! [`from_mvox_file`] loads a decoded [`MVoxFile`](mvox::MVoxFile) into one
//! and [`to_mvox_file`] writes one back, exactly for a loaded file.
//! [`to_mvox_vox_main`] gives a bare `VoxMain<()>` a synthesized ext, and
//! `take_ext` takes the ext back off. The ext keys its entries by entity id
//! and follows the state through the [`VoxExt`](voxcore::VoxExt) hooks, so a
//! state mutated after the load still writes back with the surviving
//! provenance. The `codec` module,
//! behind the default `codec` feature, goes straight to and from `.vox` bytes
//! over mvox-codec. The `serde` feature, on by default, derives serde for the
//! ext types.

// Public API

mod error;
mod ext;
mod from_mvox_file;
mod mvox_vox_main;
mod result;
mod to_mvox_file;
mod to_mvox_vox_main;

pub use error::*;
pub use ext::*;
pub use from_mvox_file::*;
pub use mvox_vox_main::*;
pub use result::*;
pub use to_mvox_file::*;
pub use to_mvox_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
