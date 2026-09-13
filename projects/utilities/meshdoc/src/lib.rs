#![deny(rustdoc::broken_intra_doc_links)]

//! Core types for working with meshes.

pub mod check;
pub mod material;

mod b_mesh_file;
mod b_mesh_hierarchy_node;
mod b_mesh_image;
mod b_mesh_material;
mod b_mesh_object;
mod b_mesh_primitive;
mod b_mesh_texture;
mod b_mesh_triangle;
mod b_mesh_uv_stream;
mod b_mesh_vertex;
mod b_mesh_vertex_attribute;
mod error;
mod mesh_alpha_mode;
mod mesh_attribute_components;
mod mesh_ext;
mod mesh_file;
mod mesh_gc_remap;
mod mesh_hierarchy_node;
mod mesh_image;
mod mesh_image_media_type;
mod mesh_image_source;
mod mesh_mag_filter;
mod mesh_main;
mod mesh_material;
mod mesh_min_filter;
mod mesh_object;
mod mesh_primitive;
mod mesh_property;
mod mesh_property_value;
mod mesh_state;
mod mesh_texture;
mod mesh_texture_ref;
mod mesh_triangle;
mod mesh_vertex_attribute;
mod mesh_wrap;
mod result;
mod taken_ext;

pub use b_mesh_file::*;
pub use b_mesh_hierarchy_node::*;
pub use b_mesh_image::*;
pub use b_mesh_material::*;
pub use b_mesh_object::*;
pub use b_mesh_primitive::*;
pub use b_mesh_texture::*;
pub use b_mesh_triangle::*;
pub use b_mesh_uv_stream::*;
pub use b_mesh_vertex::*;
pub use b_mesh_vertex_attribute::*;
pub use error::*;
pub use mesh_alpha_mode::*;
pub use mesh_attribute_components::*;
pub use mesh_ext::*;
pub use mesh_file::*;
pub use mesh_gc_remap::*;
pub use mesh_hierarchy_node::*;
pub use mesh_image::*;
pub use mesh_image_media_type::*;
pub use mesh_image_source::*;
pub use mesh_mag_filter::*;
pub use mesh_main::*;
pub use mesh_material::*;
pub use mesh_min_filter::*;
pub use mesh_object::*;
pub use mesh_primitive::*;
pub use mesh_property::*;
pub use mesh_property_value::*;
pub use mesh_state::*;
pub use mesh_texture::*;
pub use mesh_texture_ref::*;
pub use mesh_triangle::*;
pub use mesh_vertex_attribute::*;
pub use mesh_wrap::*;
pub use result::*;
pub use taken_ext::*;

// Test support.

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
