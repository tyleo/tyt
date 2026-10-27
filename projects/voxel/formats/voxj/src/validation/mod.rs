//! Validates a parsed [`VoxjFile`](crate::VoxjFile) against the format rules.
//! The geometry checks decode each object's blocks through the caller's
//! [`DecodeBase64`](crate::DecodeBase64).

// Public API

mod check_voxj_file;
mod error;
mod result;
mod validate_voxj_file;
mod voxj_check;
mod voxj_check_status;

pub use check_voxj_file::*;
pub use error::*;
pub use result::*;
pub use validate_voxj_file::*;
pub use voxj_check::*;
pub use voxj_check_status::*;

// Internal API

mod internal;

pub(crate) use internal::*;
