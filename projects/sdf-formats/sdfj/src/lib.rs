#![deny(rustdoc::broken_intra_doc_links)]

//! Core types for the SDF Json (`.sdfj`) format. A document records a model's
//! calls as JSON tables whose entries reference each other by index.
//!
//! The types are the data model, with optional `serde` support behind the
//! `serde` feature. A parse checks the keys and kinds each entry holds and
//! leaves the references to the reader. A write errors on a NaN, an infinity,
//! or a repeated object key.

// Public API

mod sdfj_axes2d;
mod sdfj_axes3d;
mod sdfj_axis;
mod sdfj_axis_rotation;
mod sdfj_border;
mod sdfj_caps;
mod sdfj_file;
mod sdfj_int_value;
mod sdfj_map;
mod sdfj_map_entry;
mod sdfj_material;
mod sdfj_names;
mod sdfj_node;
mod sdfj_object;
mod sdfj_pattern;
mod sdfj_property_value;
mod sdfj_shades;
mod sdfj_shape2d;
mod sdfj_shape3d;
mod sdfj_side;
mod sdfj_step;
mod sdfj_tagged_value;
mod sdfj_value;
mod sdfj_version;

pub use sdfj_axes2d::*;
pub use sdfj_axes3d::*;
pub use sdfj_axis::*;
pub use sdfj_axis_rotation::*;
pub use sdfj_border::*;
pub use sdfj_caps::*;
pub use sdfj_file::*;
pub use sdfj_int_value::*;
pub use sdfj_map::*;
pub use sdfj_map_entry::*;
pub use sdfj_material::*;
pub use sdfj_names::*;
pub use sdfj_node::*;
pub use sdfj_object::*;
pub use sdfj_pattern::*;
pub use sdfj_property_value::*;
pub use sdfj_shades::*;
pub use sdfj_shape2d::*;
pub use sdfj_shape3d::*;
pub use sdfj_side::*;
pub use sdfj_step::*;
pub use sdfj_tagged_value::*;
pub use sdfj_value::*;
pub use sdfj_version::*;

// Internal API

#[cfg(feature = "serde")]
mod find_non_finite;
#[cfg(feature = "serde")]
mod finite;
#[cfg(feature = "serde")]
mod is_false;
#[cfg(feature = "serde")]
mod present;

#[cfg(feature = "serde")]
pub(crate) use find_non_finite::*;
#[cfg(feature = "serde")]
pub(crate) use finite::*;
#[cfg(feature = "serde")]
pub(crate) use is_false::*;
#[cfg(feature = "serde")]
pub(crate) use present::*;
