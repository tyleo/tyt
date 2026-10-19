#![deny(rustdoc::broken_intra_doc_links)]

//! The render contract for voxel scenes.
//!
//! The contract types describe an image of a scene: the materials, the
//! lights, the views, and the output buffer. Every renderer consumes them
//! and implements [`Render`]. The shared quantization carries rays into each
//! placement's grid as integers, so every renderer walks the same rays.
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
mod grid_point;
mod grid_quantization;
mod grid_vector;
mod quantize_direction;
mod quantize_point;
mod render;
mod render_bloom;
mod render_grid_ray;
mod render_grid_view_rays;
mod render_image;
mod render_light;
mod render_material;
mod render_object;
mod render_occlusion;
mod render_output;
mod render_pixel;
mod render_placement;
mod render_projection;
mod render_ray;
mod render_scene;
mod render_shadow;
mod render_view;
mod render_view_rays;
mod result;
mod shadow_target;
mod tonemap;

pub use b_render_light::*;
pub use b_render_material::*;
pub use b_render_placement::*;
pub use b_render_view::*;
pub use error::*;
pub use fit_distance::*;
pub use fit_margin::*;
pub use fit_scale::*;
pub use grid_point::*;
pub use grid_quantization::*;
pub use grid_vector::*;
pub use quantize_direction::*;
pub use quantize_point::*;
pub use render::*;
pub use render_bloom::*;
pub use render_grid_ray::*;
pub use render_grid_view_rays::*;
pub use render_image::*;
pub use render_light::*;
pub use render_material::*;
pub use render_object::*;
pub use render_occlusion::*;
pub use render_output::*;
pub use render_pixel::*;
pub use render_placement::*;
pub use render_projection::*;
pub use render_ray::*;
pub use render_scene::*;
pub use render_shadow::*;
pub use render_view::*;
pub use render_view_rays::*;
pub use result::*;
pub use shadow_target::*;
pub use tonemap::*;

// Optional API

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

// Internal API

mod fit_radius;

pub(crate) use fit_radius::*;
