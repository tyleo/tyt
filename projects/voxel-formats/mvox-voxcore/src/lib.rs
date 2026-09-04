#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between MagicaVoxel files and the voxcore state.
//!
//! [`from_mvox_file`] loads a decoded [`MVoxFile`](mvox::MVoxFile) into a
//! [`MagicaVoxelVoxMain`], and [`to_mvox_file`] writes one back. The
//! MagicaVoxel state with no native voxcore home rides in the
//! [`MagicaVoxelExt`] on the state's ext slot, so a loaded file writes back
//! exactly. A state without one, such as one loaded from another format, has
//! its file synthesized from the bare scene. The [`codec`] module, behind the
//! default `codec` feature, goes straight to and from `.vox` bytes over
//! mvox-codec. The `ext` feature keys the ext into a document's `ext` block
//! through voxcore's [`VoxExtCodec`](voxcore::ext::VoxExtCodec).

// Public API

mod error;
mod from_mvox_file;
mod magica_voxel_camera;
mod magica_voxel_ext;
mod magica_voxel_frame;
mod magica_voxel_layer;
mod magica_voxel_material;
mod magica_voxel_node;
mod magica_voxel_node_body;
mod magica_voxel_shape_model;
mod magica_voxel_unknown_chunk;
mod magica_voxel_vox_main;
mod result;
mod to_mvox_file;

pub use error::*;
pub use from_mvox_file::*;
pub use magica_voxel_camera::*;
pub use magica_voxel_ext::*;
pub use magica_voxel_frame::*;
pub use magica_voxel_layer::*;
pub use magica_voxel_material::*;
pub use magica_voxel_node::*;
pub use magica_voxel_node_body::*;
pub use magica_voxel_shape_model::*;
pub use magica_voxel_unknown_chunk::*;
pub use magica_voxel_vox_main::*;
pub use result::*;
pub use to_mvox_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

#[cfg(feature = "ext")]
mod vox_ext_codec;
