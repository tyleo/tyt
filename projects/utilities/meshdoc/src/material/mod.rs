//! The shared metallic-roughness material vocabulary. Every converter and
//! pipeline keys material factors by these names. Another model that binds a
//! property under the same meaning uses the same key, so a bake reads across
//! without a translation table. The constants cover the modeled set.

mod color_range;
mod consts;
mod default_scalar;
mod material_property_kind;
mod material_range;
mod scalar_range;

pub use color_range::*;
pub use consts::*;
pub use default_scalar::*;
pub use material_property_kind::*;
pub use material_range::*;
pub use scalar_range::*;
