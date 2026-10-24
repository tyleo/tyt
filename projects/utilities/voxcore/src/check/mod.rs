//! The outcome of a check over a document, in one form whatever runs the
//! check, so a renderer lays out every check the same way.

// Public API

mod check_slash_free_names;
mod failed_check_count;
mod vox_check;
mod vox_check_status;

pub use check_slash_free_names::*;
pub use failed_check_count::*;
pub use vox_check::*;
pub use vox_check_status::*;
