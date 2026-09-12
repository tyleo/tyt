//! Reads and writes `.vox` file bytes, gated behind the `codec` feature.

mod from_mvox_bytes;
mod to_mvox_bytes;

pub use from_mvox_bytes::*;
pub use to_mvox_bytes::*;
