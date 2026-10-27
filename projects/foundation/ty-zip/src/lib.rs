//! Frames a deflate stream as a single-member zip archive and reads it back.
//! The caller compresses and decompresses the member, so the crate carries no
//! compression library.

// Public API

mod deflated_member;
mod wrap_deflated_member;

pub use deflated_member::*;
pub use wrap_deflated_member::*;
