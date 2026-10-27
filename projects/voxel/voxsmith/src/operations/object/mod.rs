//! Object operations.

// Public API

mod add_objects;
mod downsample_objects;
mod duplicate_objects;
mod keep_rule;
mod link_objects;
mod object_mesh;
mod object_voxels;
mod remove_objects;
mod resample_factor;
mod set_edit_bounds;
mod trim_objects;
mod unlink_objects;
mod upsample_objects;

pub use add_objects::*;
pub use downsample_objects::*;
pub use duplicate_objects::*;
pub use keep_rule::*;
pub use link_objects::*;
pub use object_mesh::*;
pub use object_voxels::*;
pub use remove_objects::*;
pub use resample_factor::*;
pub use set_edit_bounds::*;
pub use trim_objects::*;
pub use unlink_objects::*;
pub use upsample_objects::*;

// Optional API

#[cfg(feature = "render")]
mod object_render;

#[cfg(feature = "render")]
pub use object_render::*;

// Internal API

mod error_object_ext;
