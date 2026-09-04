#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Goxel files and the voxcore state.
//!
//! [`from_goxl_file`] loads a [`GoxlFile`](goxl::GoxlFile) into a
//! [`GoxelVoxMain`], and [`to_goxl_file`] writes one back. The Goxel state
//! with no native voxcore home rides in the [`GoxelExt`] on the state's ext
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
mod goxel_camera;
mod goxel_ext;
mod goxel_image;
mod goxel_layer;
mod goxel_light;
mod goxel_material;
mod goxel_preview;
mod goxel_unknown_chunk;
mod goxel_vox_main;
mod result;
mod to_goxl_file;

pub use error::*;
pub use from_goxl_file::*;
pub use goxel_camera::*;
pub use goxel_ext::*;
pub use goxel_image::*;
pub use goxel_layer::*;
pub use goxel_light::*;
pub use goxel_material::*;
pub use goxel_preview::*;
pub use goxel_unknown_chunk::*;
pub use goxel_vox_main::*;
pub use result::*;
pub use to_goxl_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
