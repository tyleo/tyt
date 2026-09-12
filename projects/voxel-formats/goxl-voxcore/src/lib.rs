#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Goxel files and the voxcore state.
//!
//! The state is a [`GoxlVoxMain`], a [`VoxMain`](voxcore::VoxMain) carrying a
//! [`GoxlExt`] with the Goxel state that has no native voxcore home.
//! [`from_goxl_file`] loads a [`GoxlFile`](goxl::GoxlFile) into one and
//! [`to_goxl_file`] writes one back, exactly for a loaded file.
//! [`to_goxl_vox_main`] gives a bare `VoxMain<()>` a synthesized ext, and
//! `take_ext` takes the ext back off. The ext follows the listings through
//! the [`VoxExt`](voxcore::VoxExt) hooks, so a state mutated after the load
//! still writes back with the surviving provenance. The `codec` module,
//! behind the default `codec` feature, goes straight to and from `.gox`
//! bytes. It takes the codec's dependencies, which
//! `goxl_codec::DependenciesImpl` supplies. The `serde` feature, on by
//! default, derives serde for the ext types.

// Public API

mod error;
mod ext;
mod from_goxl_file;
mod goxl_vox_main;
mod result;
mod to_goxl_file;
mod to_goxl_vox_main;

pub use error::*;
pub use ext::*;
pub use from_goxl_file::*;
pub use goxl_vox_main::*;
pub use result::*;
pub use to_goxl_file::*;
pub use to_goxl_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Internal API

mod internal;
pub(crate) use internal::*;
