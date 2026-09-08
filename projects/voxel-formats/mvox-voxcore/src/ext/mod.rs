//! The MagicaVoxel state with no native voxcore home, kept beside a loaded
//! file's scene so the file writes back exactly. The bare converters
//! synthesize the file from the scene instead. The `ext` feature makes this
//! module public and adds the typed path: [`MVoxVoxMain`] carries a
//! [`MVoxExt`] as its ext, [`from_mvox_file_with_ext`] keeps a loaded file's,
//! and [`to_mvox_file_with_ext`] writes it back. The ext enters a document's
//! `ext` block as the `mvox` entry through voxcore's
//! [`VoxExtEntryCodec`](voxcore::ext::VoxExtEntryCodec).

// Types

mod mvox_ext;
mod mvox_ext_camera;
mod mvox_ext_frame;
mod mvox_ext_layer;
mod mvox_ext_material;
mod mvox_ext_node;
mod mvox_ext_node_body;
mod mvox_ext_shape_model;
mod mvox_ext_unknown_chunk;

pub use mvox_ext::*;
pub use mvox_ext_camera::*;
pub use mvox_ext_frame::*;
pub use mvox_ext_layer::*;
pub use mvox_ext_material::*;
pub use mvox_ext_node::*;
pub use mvox_ext_node_body::*;
pub use mvox_ext_shape_model::*;
pub use mvox_ext_unknown_chunk::*;

// Typed path

#[cfg(feature = "ext")]
mod from_mvox_file_with_ext;

#[cfg(feature = "ext")]
mod mvox_vox_main;

#[cfg(feature = "ext")]
mod to_mvox_file_with_ext;

#[cfg(feature = "ext")]
mod vox_ext;

#[cfg(feature = "ext")]
mod vox_ext_entry_codec;

#[cfg(feature = "ext")]
pub use from_mvox_file_with_ext::*;

#[cfg(feature = "ext")]
pub use mvox_vox_main::*;

#[cfg(feature = "ext")]
pub use to_mvox_file_with_ext::*;
