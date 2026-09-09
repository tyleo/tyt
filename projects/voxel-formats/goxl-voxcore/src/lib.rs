#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Goxel files and the voxcore state.
//!
//! [`from_goxl_file`] loads a [`GoxlFile`](goxl::GoxlFile) into a bare
//! [`VoxMain`](voxcore::VoxMain). [`to_goxl_file`] writes one back as a file
//! synthesized from the scene. The `codec` module, behind the default `codec`
//! feature, goes straight to and from `.gox` bytes. It takes the codec's
//! dependencies, which `goxl_codec::DependenciesImpl` supplies. The `ext`
//! feature, on by default, opens the `ext` module. There the Goxel state with
//! no native voxcore home rides as the state's ext. The `_with_ext`
//! converters write a loaded file back exactly. The ext follows the listings
//! through the [`VoxExt`](voxcore::ext::VoxExt) hooks, so a state mutated
//! after the load still writes back with the surviving provenance.

// Public API

mod error;
mod from_goxl_file;
mod result;
mod to_goxl_file;

pub use error::*;
pub use from_goxl_file::*;
pub use result::*;
pub use to_goxl_file::*;

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
