//! The Voxel Json `ext` block as voxconv's exts. A block decodes into a
//! [`CompositeVoxExt`] through [`composite_vox_ext_from_voxj_vox_ext`], each
//! entry under an enabled format's key as that format's ext and the rest as
//! an [`InertVoxExt`]. [`voxj_vox_ext_from_ext`] encodes any of voxconv's
//! exts back to the block.

mod composite_vox_ext;
mod composite_vox_ext_from_voxj_vox_ext;
mod inert_vox_ext;
mod voxj_format_ext;
mod voxj_vox_ext_from_ext;

pub use composite_vox_ext::*;
pub use composite_vox_ext_from_voxj_vox_ext::*;
pub use inert_vox_ext::*;
pub use voxj_vox_ext_from_ext::*;
