//! The outcome of a check over a document. The core defines the vocabulary
//! only. A format's crate runs its checks over its encoding and reports them
//! in this form, so a renderer can lay any format's checks out the same way.

mod failed_check_count;
mod vox_check;
mod vox_check_status;

pub use failed_check_count::*;
pub use vox_check::*;
pub use vox_check_status::*;
