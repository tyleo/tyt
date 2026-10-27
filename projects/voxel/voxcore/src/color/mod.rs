//! Color reads over the voxcore types, gated behind the `color` feature.
//! [`lin_srgba_f64_from_srgba_u8()`] and [`srgba_u8_from_lin_srgba_f64()`] are
//! the one sRGB transfer applied at an 8-bit boundary: a codec decodes and
//! encodes palette colors through it, and a mesh pipeline decodes texture
//! texels and encodes atlas texels through it. [`ColorValues`] decodes a value
//! pool's values to colors once.
//! [`resolve_cell_color()`] resolves an object's `baseColor` supplier once into
//! a [`CellColor`], which a writer then reads per voxel.

// Public API

mod cell_color;
mod color_values;
mod lin_srgba_f64_from_srgba_u8;
mod resolve_cell_color;
mod resolve_cell_color_or_transparent;
mod srgba_u8_from_lin_srgba_f64;

pub use cell_color::*;
pub use color_values::*;
pub use lin_srgba_f64_from_srgba_u8::*;
pub use resolve_cell_color::*;
pub use resolve_cell_color_or_transparent::*;
pub use srgba_u8_from_lin_srgba_f64::*;
