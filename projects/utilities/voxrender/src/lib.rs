#![deny(rustdoc::broken_intra_doc_links)]

//! The render contract for voxel scenes and its CPU reference renderer.
//!
//! The contract types describe an image of a scene: the materials, the
//! lights, the views, and the output buffer. Every renderer consumes them.
//! The CPU renderer behind the default `cpu` feature follows the contract
//! literally and makes the images every other renderer is tested against.
//! Every entity has a brand, so nothing is addressed by a bare integer. The
//! only integers are pixel coordinates.

// Public API

mod b_render_light;
mod b_render_material;
mod b_render_placement;
mod b_render_view;
mod error;
mod fit_distance;
mod fit_margin;
mod fit_scale;
mod grid_quantization;
mod render_bloom;
mod render_image;
mod render_light;
mod render_material;
mod render_object;
mod render_occlusion;
mod render_output;
mod render_pixel;
mod render_placement;
mod render_projection;
mod render_scene;
mod render_shadow;
mod render_view;
mod result;
mod tonemap;

pub use b_render_light::*;
pub use b_render_material::*;
pub use b_render_placement::*;
pub use b_render_view::*;
pub use error::*;
pub use fit_distance::*;
pub use fit_margin::*;
pub use fit_scale::*;
pub use grid_quantization::*;
pub use render_bloom::*;
pub use render_image::*;
pub use render_light::*;
pub use render_material::*;
pub use render_object::*;
pub use render_occlusion::*;
pub use render_output::*;
pub use render_pixel::*;
pub use render_placement::*;
pub use render_projection::*;
pub use render_scene::*;
pub use render_shadow::*;
pub use render_view::*;
pub use result::*;
pub use tonemap::*;

// Optional API

#[cfg(feature = "cpu")]
mod cast_ray;

#[cfg(feature = "cpu")]
pub use cast_ray::*;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(feature = "cpu")]
mod render;

#[cfg(feature = "cpu")]
pub use render::*;

#[cfg(feature = "cpu")]
mod render_grid_ray;

#[cfg(feature = "cpu")]
pub use render_grid_ray::*;

#[cfg(feature = "cpu")]
mod render_grid_view_rays;

#[cfg(feature = "cpu")]
pub use render_grid_view_rays::*;

#[cfg(feature = "cpu")]
mod render_hit;

#[cfg(feature = "cpu")]
pub use render_hit::*;

#[cfg(feature = "cpu")]
mod render_ray;

#[cfg(feature = "cpu")]
pub use render_ray::*;

#[cfg(feature = "cpu")]
mod render_ray_walk;

#[cfg(feature = "cpu")]
pub use render_ray_walk::*;

#[cfg(feature = "cpu")]
mod render_view_rays;

#[cfg(feature = "cpu")]
pub use render_view_rays::*;

// Internal API

mod fit_radius;

pub(crate) use fit_radius::*;

#[cfg(feature = "cpu")]
mod bloom;

#[cfg(feature = "cpu")]
pub(crate) use bloom::*;

#[cfg(feature = "cpu")]
mod grid_point;

#[cfg(feature = "cpu")]
pub(crate) use grid_point::*;

#[cfg(feature = "cpu")]
mod grid_vector;

#[cfg(feature = "cpu")]
pub(crate) use grid_vector::*;

#[cfg(feature = "cpu")]
mod is_opaque;

#[cfg(feature = "cpu")]
pub(crate) use is_opaque::*;

#[cfg(feature = "cpu")]
mod material_pass;

#[cfg(feature = "cpu")]
pub(crate) use material_pass::*;

#[cfg(feature = "cpu")]
mod normal_reflectance;

#[cfg(feature = "cpu")]
pub(crate) use normal_reflectance::*;

#[cfg(feature = "cpu")]
mod placement_bounds;

#[cfg(feature = "cpu")]
pub(crate) use placement_bounds::*;

#[cfg(feature = "cpu")]
mod quantize_direction;

#[cfg(feature = "cpu")]
pub(crate) use quantize_direction::*;

#[cfg(feature = "cpu")]
mod quantize_point;

#[cfg(feature = "cpu")]
pub(crate) use quantize_point::*;

#[cfg(feature = "cpu")]
mod render_grid;

#[cfg(feature = "cpu")]
pub(crate) use render_grid::*;

#[cfg(feature = "cpu")]
mod shadow_target;

#[cfg(feature = "cpu")]
pub(crate) use shadow_target::*;

#[cfg(feature = "cpu")]
mod transmitted_reflectance;

#[cfg(feature = "cpu")]
pub(crate) use transmitted_reflectance::*;

// Test support

#[cfg(all(test, feature = "cpu"))]
mod test_utilities;
