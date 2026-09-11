//! The Goxel state with no native voxcore home, kept beside a loaded file's
//! scene so the file writes back exactly. The bare converters synthesize the
//! file from the scene instead. The typed path keeps it: [`GoxlVoxMain`]
//! carries a [`GoxlExt`] as its ext, [`from_goxl_file_with_ext`] keeps a loaded
//! file's, and [`to_goxl_file_with_ext`] writes it back. The ext follows the
//! state's listings through voxcore's [`VoxExt`](voxcore::VoxExt) hooks. The
//! `serde` feature derives serde for the types here.

// Types

mod goxl_ext;
mod goxl_ext_camera;
mod goxl_ext_image;
mod goxl_ext_layer;
mod goxl_ext_light;
mod goxl_ext_material;
mod goxl_ext_preview;
mod goxl_ext_unknown_chunk;

pub use goxl_ext::*;
pub use goxl_ext_camera::*;
pub use goxl_ext_image::*;
pub use goxl_ext_layer::*;
pub use goxl_ext_light::*;
pub use goxl_ext_material::*;
pub use goxl_ext_preview::*;
pub use goxl_ext_unknown_chunk::*;

// Typed path

mod from_goxl_file_with_ext;

mod goxl_vox_main;

mod to_goxl_file_with_ext;

pub use from_goxl_file_with_ext::*;

pub use goxl_vox_main::*;

pub use to_goxl_file_with_ext::*;
