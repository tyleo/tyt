#![deny(rustdoc::broken_intra_doc_links)]

//! Core types for signed distance field models.
//!
//! An [`SdfState`] holds a model's tables, one per kind of value a model can
//! share. Entries reference each other by branded id. [`SdfMain::new`]
//! validates a state against the rules a document follows. A reader of an
//! [`SdfMain`] can trust every id it meets.

// Public API

mod b_sdf_material;
mod b_sdf_node;
mod b_sdf_object;
mod b_sdf_pattern;
mod b_sdf_shades;
mod b_sdf_shape2d;
mod b_sdf_shape3d;
mod b_sdf_step;
mod error;
mod result;
mod sdf_axes2d;
mod sdf_axes3d;
mod sdf_axis_rotation;
mod sdf_caps;
mod sdf_entry_id;
mod sdf_main;
mod sdf_map;
mod sdf_map_entry;
mod sdf_material;
mod sdf_names;
mod sdf_node;
mod sdf_object;
mod sdf_pattern;
mod sdf_property;
mod sdf_property_value;
mod sdf_shades;
mod sdf_shape2d;
mod sdf_shape3d;
mod sdf_side;
mod sdf_state;
mod sdf_step;
mod sdf_step_material;
mod sdf_value;

pub use b_sdf_material::*;
pub use b_sdf_node::*;
pub use b_sdf_object::*;
pub use b_sdf_pattern::*;
pub use b_sdf_shades::*;
pub use b_sdf_shape2d::*;
pub use b_sdf_shape3d::*;
pub use b_sdf_step::*;
pub use error::*;
pub use result::*;
pub use sdf_axes2d::*;
pub use sdf_axes3d::*;
pub use sdf_axis_rotation::*;
pub use sdf_caps::*;
pub use sdf_entry_id::*;
pub use sdf_main::*;
pub use sdf_map::*;
pub use sdf_map_entry::*;
pub use sdf_material::*;
pub use sdf_names::*;
pub use sdf_node::*;
pub use sdf_object::*;
pub use sdf_pattern::*;
pub use sdf_property::*;
pub use sdf_property_value::*;
pub use sdf_shades::*;
pub use sdf_shape2d::*;
pub use sdf_shape3d::*;
pub use sdf_side::*;
pub use sdf_state::*;
pub use sdf_step::*;
pub use sdf_step_material::*;
pub use sdf_value::*;
