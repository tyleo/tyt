//! Reads and writes Goxel (`.gox`) files.
//!
//! [`from_gox_file_bytes`] parses bytes into a [`goxl::GoxlFile`];
//! [`to_gox_file_bytes`] writes one back to an equivalent file;
//! [`validate_gox_file`] optionally checks a file for shape and cross-reference
//! faults. The shared `BL16` voxel blocks and the `PREV` preview are PNGs,
//! decoded and encoded through the caller's [`DecodePng`] and [`EncodePng`],
//! which `DependenciesImpl` supplies behind the default `impl` feature. See
//! the `goxl` crate for the data types.

// Public API

mod decode_png;
mod encode_png;
mod error;
mod from_gox_file_bytes;
mod goxl_rgba_image;
mod result;
mod to_gox_file_bytes;
mod validate_gox_file;

pub use decode_png::*;
pub use encode_png::*;
pub use error::*;
pub use from_gox_file_bytes::*;
pub use goxl_rgba_image::*;
pub use result::*;
pub use to_gox_file_bytes::*;
pub use validate_gox_file::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;

// Internal API

mod internal;
pub(crate) use internal::*;
