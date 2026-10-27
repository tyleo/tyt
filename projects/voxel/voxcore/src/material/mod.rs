//! The shared metallic-roughness material vocabulary. Every converter, mesh
//! pipeline, and CLI binds and reads palette properties by these names. One
//! table keeps producers and consumers in agreement. A property name stays a
//! free string, which keeps a format's custom properties expressible. The
//! constants cover only the recommended set. [`MaterialPropertyKind`]
//! classifies a name and [`default_scalar()`] gives a scalar name its standard
//! default. Under the `color` feature, [`pass()`] takes a material's values
//! and gives the light its surface lets through. The renderer and the mesher
//! both read opacity from it.

// Public API

mod consts;
mod default_scalar;
mod material_property_kind;

pub use consts::*;
pub use default_scalar::*;
pub use material_property_kind::*;

// Optional API

#[cfg(feature = "color")]
mod normal_reflectance;

#[cfg(feature = "color")]
pub use normal_reflectance::*;

#[cfg(feature = "color")]
mod pass;

#[cfg(feature = "color")]
pub use pass::*;
