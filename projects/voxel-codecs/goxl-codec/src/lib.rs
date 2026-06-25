//! Reads and writes Goxel (`.gox`) files.
//!
//! [`from_gox_file_bytes`] parses bytes into a [`goxl::GoxlFile`];
//! [`to_gox_file_bytes`] writes one back to an equivalent file;
//! [`validate_gox_file`] optionally checks a file for shape and cross-reference
//! faults. See the `goxl` crate for the data types.

mod internal;
pub(crate) use internal::*;

mod error;
mod from_gox_file_bytes;
mod result;
mod to_gox_file_bytes;
mod validate_gox_file;

pub use error::*;
pub use from_gox_file_bytes::*;
pub use result::*;
pub use to_gox_file_bytes::*;
pub use validate_gox_file::*;
