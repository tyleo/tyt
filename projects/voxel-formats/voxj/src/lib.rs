#![deny(rustdoc::broken_intra_doc_links)]

// Public API

pub mod objects;
pub mod validation;

mod cost_voxj_object;
mod decode_base64;
mod encode_base64;
mod voxj_edit_object;
mod voxj_edit_state;
mod voxj_file;
mod voxj_hierarchy_node;
mod voxj_main;
mod voxj_map;
mod voxj_object;
mod voxj_palette;
mod voxj_position_block;
mod voxj_property;
mod voxj_runtime_state;
mod voxj_sample_block;
mod voxj_transform;
mod voxj_value;
mod voxj_value_pool;

pub use cost_voxj_object::*;
pub use decode_base64::*;
pub use encode_base64::*;
pub use voxj_edit_object::*;
pub use voxj_edit_state::*;
pub use voxj_file::*;
pub use voxj_hierarchy_node::*;
pub use voxj_main::*;
pub use voxj_map::*;
pub use voxj_object::*;
pub use voxj_palette::*;
pub use voxj_position_block::*;
pub use voxj_property::*;
pub use voxj_runtime_state::*;
pub use voxj_sample_block::*;
pub use voxj_transform::*;
pub use voxj_value::*;
pub use voxj_value_pool::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
