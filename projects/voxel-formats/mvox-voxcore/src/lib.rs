#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between MagicaVoxel files and the voxcore state.
//!
//! [`from_mvox_file`] loads a decoded [`MVoxFile`](mvox::MVoxFile) into a
//! [`MVoxVoxMain`], and [`to_mvox_file`] writes one back. The
//! MagicaVoxel state with no native voxcore home rides in the
//! [`MVoxExt`] on the state's ext slot, so a loaded file writes back
//! exactly. A state without one, such as one loaded from another format, has
//! its file synthesized from the bare scene. The [`codec`] module, behind the
//! default `codec` feature, goes straight to and from `.vox` bytes over
//! mvox-codec. The `ext` feature keys the ext into a document's `ext` block
//! through voxcore's [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_mvox_file;
mod mvox_ext;
mod mvox_ext_camera;
mod mvox_ext_frame;
mod mvox_ext_layer;
mod mvox_ext_material;
mod mvox_ext_node;
mod mvox_ext_node_body;
mod mvox_ext_shape_model;
mod mvox_ext_unknown_chunk;
mod mvox_vox_main;
mod result;
mod to_mvox_file;

pub use error::*;
pub use from_mvox_file::*;
pub use mvox_ext::*;
pub use mvox_ext_camera::*;
pub use mvox_ext_frame::*;
pub use mvox_ext_layer::*;
pub use mvox_ext_material::*;
pub use mvox_ext_node::*;
pub use mvox_ext_node_body::*;
pub use mvox_ext_shape_model::*;
pub use mvox_ext_unknown_chunk::*;
pub use mvox_vox_main::*;
pub use result::*;
pub use to_mvox_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
