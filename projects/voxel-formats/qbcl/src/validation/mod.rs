//! Checks a decoded file for the count and size mismatches its byte layout
//! cannot catch. Decoding always produces grids of the right length, but a
//! hand-built or edited file can hold a grid whose length disagrees with its
//! declared size. The codec writes such a file as structurally valid but
//! broken bytes. Decoding does not run these checks. Run one when you need
//! the guarantee.

mod error;
mod result;
#[cfg(feature = "qb")]
mod validate_qb_file;
#[cfg(feature = "qbcl")]
mod validate_qbcl_file;
#[cfg(feature = "qbt")]
mod validate_qbt_file;

pub use error::*;
pub use result::*;
#[cfg(feature = "qb")]
pub use validate_qb_file::*;
#[cfg(feature = "qbcl")]
pub use validate_qbcl_file::*;
#[cfg(feature = "qbt")]
pub use validate_qbt_file::*;
