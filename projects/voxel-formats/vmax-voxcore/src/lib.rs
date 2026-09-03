#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Max packages and the voxcore state.
//!
//! [`from_vmax_file`] loads a [`VMaxFile`](vmax::VMaxFile) into a
//! [`VoxelMaxVoxMain`], and [`to_vmax_file`] writes one back, with
//! [`VmaxFileBuilder`] for control over the color format and the scene
//! camera. The Voxel Max state with no native voxcore home rides in the
//! [`VoxelMaxExt`] on the state's ext slot, so a loaded document writes back
//! exactly. A state without one, such as one loaded from another format, has
//! its document synthesized from the bare scene. The [`codec`] module, behind
//! the default `codec` feature, goes straight to and from a package's files.
//! The `ext` feature keys the ext into a document's `ext` block through
//! voxcore's [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_vmax_file;
mod result;
mod scene_camera_source;
mod to_vmax_file;
mod vmax_file_builder;
mod voxel_max_color_format;
mod voxel_max_ext;
mod voxel_max_material;
mod voxel_max_material_dispersion;
mod voxel_max_node;
mod voxel_max_object_state;
mod voxel_max_palette;
mod voxel_max_vox_main;

pub use error::*;
pub use from_vmax_file::*;
pub use result::*;
pub use scene_camera_source::*;
pub use to_vmax_file::*;
pub use vmax_file_builder::*;
pub use voxel_max_color_format::*;
pub use voxel_max_ext::*;
pub use voxel_max_material::*;
pub use voxel_max_material_dispersion::*;
pub use voxel_max_node::*;
pub use voxel_max_object_state::*;
pub use voxel_max_palette::*;
pub use voxel_max_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;

// Internal API

mod internal;
pub(crate) use internal::*;
