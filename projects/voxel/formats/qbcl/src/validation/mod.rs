//! Checks a decoded file for the count and size mismatches its byte layout
//! cannot catch. Decoding always produces grids of the right length, but a
//! hand-built or edited file can hold a grid whose length disagrees with its
//! declared size. The codec writes such a file as structurally valid but
//! broken bytes. Decoding does not run these checks. Run one when you need
//! the guarantee.

// Public API

mod error;
mod result;

pub use error::*;
pub use result::*;

// Optional API

#[cfg(feature = "qb")]
mod validate_qb_file;

#[cfg(feature = "qb")]
pub use validate_qb_file::*;

#[cfg(feature = "qbcl")]
mod validate_qbcl_file;

#[cfg(feature = "qbcl")]
pub use validate_qbcl_file::*;

#[cfg(feature = "qbt")]
mod validate_qbt_file;

#[cfg(feature = "qbt")]
pub use validate_qbt_file::*;

// Internal API

#[cfg(any(feature = "qb", feature = "qbcl", feature = "qbt"))]
mod grid_cell_count;

#[cfg(any(feature = "qb", feature = "qbcl", feature = "qbt"))]
pub(crate) use grid_cell_count::*;
