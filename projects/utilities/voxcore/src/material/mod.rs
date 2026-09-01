//! The shared metallic-roughness material vocabulary. Every converter, mesh
//! pipeline, and CLI binds and reads palette properties by these names. One
//! table keeps producers and consumers in agreement. A property name stays a
//! free string, which keeps a format's custom properties expressible. The
//! constants cover only the recommended set. [`MaterialPropertyKind`] classifies a
//! name and [`default_scalar`] gives a scalar name its standard default.

mod consts;
mod default_scalar;
mod material_property_kind;

pub use consts::*;
pub use default_scalar::*;
pub use material_property_kind::*;
