#![deny(rustdoc::broken_intra_doc_links)]

//! The CPU reference renderer for `voxrender`'s render contract. It follows
//! the contract literally and makes the images every other renderer is
//! tested against.

// Public API

mod cast_ray;
mod dependencies;
mod map_rows;
mod render;
mod render_hit;
mod render_ray_walk;
mod shadow_rays;

pub use cast_ray::*;
pub use dependencies::*;
pub use map_rows::*;
pub use render::*;
pub use render_hit::*;
pub use render_ray_walk::*;
pub use shadow_rays::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

// Internal API

mod bloom;
mod is_opaque;
mod material_pass;
mod normal_reflectance;
mod placement_bounds;
mod render_grid;
mod shadow_origin;
mod transmitted_reflectance;

pub(crate) use bloom::*;
pub(crate) use is_opaque::*;
pub(crate) use material_pass::*;
pub(crate) use normal_reflectance::*;
pub(crate) use placement_bounds::*;
pub(crate) use render_grid::*;
pub(crate) use shadow_origin::*;
pub(crate) use transmitted_reflectance::*;

// Test support

#[cfg(test)]
mod test_utilities;
