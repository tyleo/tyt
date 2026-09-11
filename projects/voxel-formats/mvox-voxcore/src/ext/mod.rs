//! The MagicaVoxel state with no native voxcore home, kept beside a loaded
//! file's scene so the file writes back exactly. The bare converters synthesize
//! the file from the scene instead. The typed path keeps it: [`MVoxVoxMain`]
//! carries a [`MVoxExt`] as its ext, [`from_mvox_file_with_ext`] keeps a loaded
//! file's, and [`to_mvox_file_with_ext`] writes it back. The ext follows the
//! state's listings through voxcore's [`VoxExt`](voxcore::VoxExt) hooks. The
//! `serde` feature derives serde for the types here.

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

mod from_mvox_file_with_ext;

mod mvox_vox_main;

mod to_mvox_file_with_ext;

pub use from_mvox_file_with_ext::*;

pub use mvox_vox_main::*;

pub use to_mvox_file_with_ext::*;
