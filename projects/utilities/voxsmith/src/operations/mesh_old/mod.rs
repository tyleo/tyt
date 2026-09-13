// Public API

mod atlas_image;
mod atlas_shape;
mod color_channel;
mod material_atlas;
mod material_bake;
mod material_channel;
mod material_map;
mod material_mesh_request;
mod material_slot;
mod mesh_geometry;
mod mesh_method;
#[allow(clippy::module_inception)]
mod mesh_old;
mod object_to_mesh_geometry;

pub use atlas_image::*;
pub use atlas_shape::*;
pub use color_channel::*;
pub use material_atlas::*;
pub use material_bake::*;
pub use material_channel::*;
pub use material_map::*;
pub use material_mesh_request::*;
pub use material_slot::*;
pub use mesh_geometry::*;
pub use mesh_method::*;
pub use mesh_old::*;
pub use object_to_mesh_geometry::*;

// Internal API

mod internal;
pub(crate) use internal::*;
