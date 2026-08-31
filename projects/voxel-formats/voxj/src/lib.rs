#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "objects")]
pub mod objects;

#[cfg(feature = "validation")]
pub mod validation;

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
