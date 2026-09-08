//! Reads and writes `.vox` file bytes, gated behind the `codec` feature. With
//! the `ext` feature, `from_mvox_bytes_with_ext` and `to_mvox_bytes_with_ext`
//! move the bytes through the typed path.

mod from_mvox_bytes;
mod to_mvox_bytes;

pub use from_mvox_bytes::*;
pub use to_mvox_bytes::*;

#[cfg(feature = "ext")]
mod from_mvox_bytes_with_ext;

#[cfg(feature = "ext")]
mod to_mvox_bytes_with_ext;

#[cfg(feature = "ext")]
pub use from_mvox_bytes_with_ext::*;

#[cfg(feature = "ext")]
pub use to_mvox_bytes_with_ext::*;
