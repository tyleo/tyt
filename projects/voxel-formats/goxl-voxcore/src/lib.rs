#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Goxel files and the voxcore state.
//!
//! [`from_goxl_file`] loads a [`GoxlFile`](goxl::GoxlFile) into a
//! [`GoxlVoxMain`], and [`to_goxl_file`] writes one back. The Goxel state
//! with no native voxcore home rides in the [`GoxlExt`] on the state's ext
//! slot, so a loaded file writes back exactly. A state without one, such as
//! one loaded from another format, has its file synthesized from the bare
//! scene. The [`codec`] module, behind the default `codec` feature, goes
//! straight to and from `.gox` bytes. It takes the codec's dependencies,
//! which `goxl_codec::DependenciesImpl` supplies. The `ext` feature keys the
//! ext into a document's `ext` block through voxcore's
//! [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_goxl_file;
mod goxl_ext;
mod goxl_ext_camera;
mod goxl_ext_image;
mod goxl_ext_layer;
mod goxl_ext_light;
mod goxl_ext_material;
mod goxl_ext_preview;
mod goxl_ext_unknown_chunk;
mod goxl_vox_main;
mod result;
mod to_goxl_file;

pub use error::*;
pub use from_goxl_file::*;
pub use goxl_ext::*;
pub use goxl_ext_camera::*;
pub use goxl_ext_image::*;
pub use goxl_ext_layer::*;
pub use goxl_ext_light::*;
pub use goxl_ext_material::*;
pub use goxl_ext_preview::*;
pub use goxl_ext_unknown_chunk::*;
pub use goxl_vox_main::*;
pub use result::*;
pub use to_goxl_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
