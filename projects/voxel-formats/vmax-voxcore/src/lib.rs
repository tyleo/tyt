#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Max packages and the voxcore state.
//!
//! [`from_vmax_file`] loads a [`VMaxFile`](vmax::VMaxFile) into a bare
//! [`VoxMain`](voxcore::VoxMain). [`to_vmax_file`] writes one back as a
//! document synthesized from the scene, with [`VMaxWriteOptions`] picking
//! the color format and the scene camera. The `codec` module, behind the
//! default `codec` feature, goes straight to and from a package's files. It
//! takes the codec's dependencies, which `vmax_codec::DependenciesImpl`
//! supplies. The `ext` feature, on by default, opens the `ext` module. There
//! the Voxel Max state with no native voxcore home rides as the state's ext.
//! The `_with_ext` converters write a loaded document back exactly.

// Public API

mod error;
mod from_vmax_file;
mod result;
mod scene_camera_source;
mod to_vmax_file;
mod vmax_color_format;
mod vmax_write_options;

pub use error::*;
pub use from_vmax_file::*;
pub use result::*;
pub use scene_camera_source::*;
pub use to_vmax_file::*;
pub use vmax_color_format::*;
pub use vmax_write_options::*;

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
