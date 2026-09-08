//! The Voxel Max state with no native voxcore home, kept beside a loaded
//! document's scene so the document writes back exactly. The bare converters
//! synthesize it from the scene instead. The `ext` feature makes this module
//! public and adds the typed path: [`VMaxVoxMain`] carries a [`VMaxExt`] as
//! its ext, [`from_vmax_file_with_ext`] keeps a loaded document's, and
//! [`to_vmax_file_with_ext`] writes it back. The ext follows the state's
//! listings through voxcore's [`VoxExt`](voxcore::ext::VoxExt) hooks. Its
//! entry in a document's `ext` block is `vmax`, through
//! [`VoxExtEntryCodec`](voxcore::ext::VoxExtEntryCodec).

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

#[cfg(feature = "ext")]
mod from_vmax_file_with_ext;

#[cfg(feature = "ext")]
mod to_vmax_file_with_ext;

#[cfg(feature = "ext")]
mod vmax_vox_main;

#[cfg(feature = "ext")]
mod vox_ext;

#[cfg(feature = "ext")]
mod vox_ext_entry_codec;

#[cfg(feature = "ext")]
pub use from_vmax_file_with_ext::*;

#[cfg(feature = "ext")]
pub use to_vmax_file_with_ext::*;

#[cfg(feature = "ext")]
pub use vmax_vox_main::*;
