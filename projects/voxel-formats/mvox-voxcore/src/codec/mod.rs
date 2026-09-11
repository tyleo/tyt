//! Reads and writes `.vox` file bytes, gated behind the `codec` feature.
//! `from_mvox_bytes_with_ext` and `to_mvox_bytes_with_ext` move the bytes
//! through the typed path.

mod from_mvox_bytes;
mod to_mvox_bytes;

pub use from_mvox_bytes::*;
pub use to_mvox_bytes::*;

mod from_mvox_bytes_with_ext;

mod to_mvox_bytes_with_ext;

pub use from_mvox_bytes_with_ext::*;

pub use to_mvox_bytes_with_ext::*;
