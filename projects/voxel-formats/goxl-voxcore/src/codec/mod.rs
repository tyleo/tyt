//! Reads and writes `.gox` file bytes, gated behind the `codec` feature. With
//! the `ext` feature, `from_goxl_bytes_with_ext` and `to_goxl_bytes_with_ext`
//! move the bytes through the typed path.

mod from_goxl_bytes;
mod to_goxl_bytes;

pub use from_goxl_bytes::*;
pub use to_goxl_bytes::*;

#[cfg(feature = "ext")]
mod from_goxl_bytes_with_ext;

#[cfg(feature = "ext")]
mod to_goxl_bytes_with_ext;

#[cfg(feature = "ext")]
pub use from_goxl_bytes_with_ext::*;

#[cfg(feature = "ext")]
pub use to_goxl_bytes_with_ext::*;

// Re-exported so a caller can name the dependencies the functions here take
// and bind them through the codec's impl behind `impl`.
pub use goxl_codec::dependencies;
