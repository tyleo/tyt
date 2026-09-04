#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between Voxel Max packages and the voxcore state.
//!
//! [`from_vmax_file`] loads a [`VMaxFile`](vmax::VMaxFile) into a
//! [`VMaxVoxMain`], and [`to_vmax_file`] writes one back, with
//! [`VmaxFileBuilder`] for control over the color format and the scene
//! camera. The Voxel Max state with no native voxcore home rides in the
//! [`VMaxExt`] on the state's ext slot, so a loaded document writes back
//! exactly. A state without one, such as one loaded from another format, has
//! its document synthesized from the bare scene. The [`codec`] module, behind
//! the default `codec` feature, goes straight to and from a package's files.
//! It takes the codec's dependencies, which `vmax_codec::DependenciesImpl`
//! supplies. The `ext` feature keys the ext into a document's `ext` block
//! through voxcore's [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_vmax_file;
mod result;
mod scene_camera_source;
mod to_vmax_file;
mod vmax_color_format;
mod vmax_ext;
mod vmax_ext_material;
mod vmax_ext_material_dispersion;
mod vmax_ext_node;
mod vmax_ext_object_state;
mod vmax_ext_palette;
mod vmax_file_builder;
mod vmax_vox_main;

pub use error::*;
pub use from_vmax_file::*;
pub use result::*;
pub use scene_camera_source::*;
pub use to_vmax_file::*;
pub use vmax_color_format::*;
pub use vmax_ext::*;
pub use vmax_ext_material::*;
pub use vmax_ext_material_dispersion::*;
pub use vmax_ext_node::*;
pub use vmax_ext_object_state::*;
pub use vmax_ext_palette::*;
pub use vmax_file_builder::*;
pub use vmax_vox_main::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;

// Internal API

mod internal;
pub(crate) use internal::*;
