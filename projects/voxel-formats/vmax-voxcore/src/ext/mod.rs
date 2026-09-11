//! The Voxel Max state with no native voxcore home, kept beside a loaded
//! document's scene so the document writes back exactly. The bare converters
//! synthesize it from the scene instead. The typed path keeps it:
//! [`VMaxVoxMain`] carries a [`VMaxExt`] as its ext,
//! [`from_vmax_file_with_ext`] keeps a loaded document's, and
//! [`to_vmax_file_with_ext`] writes it back. The ext follows the state's
//! listings through voxcore's [`VoxExt`](voxcore::VoxExt) hooks. The `serde`
//! feature derives serde for the types here.

// Types

mod vmax_ext;
mod vmax_ext_material;
mod vmax_ext_material_dispersion;
mod vmax_ext_node;
mod vmax_ext_object_state;
mod vmax_ext_palette;

pub use vmax_ext::*;
pub use vmax_ext_material::*;
pub use vmax_ext_material_dispersion::*;
pub use vmax_ext_node::*;
pub use vmax_ext_object_state::*;
pub use vmax_ext_palette::*;

// Typed path

mod from_vmax_file_with_ext;

mod to_vmax_file_with_ext;

mod vmax_vox_main;

pub use from_vmax_file_with_ext::*;

pub use to_vmax_file_with_ext::*;

pub use vmax_vox_main::*;
