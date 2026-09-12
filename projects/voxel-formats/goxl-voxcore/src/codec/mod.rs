//! Reads and writes `.gox` file bytes, gated behind the `codec` feature.

mod from_goxl_bytes;
mod to_goxl_bytes;

pub use from_goxl_bytes::*;
pub use to_goxl_bytes::*;

// Re-exported so a caller can name the dependencies the functions here take
// and bind them through the codec's impl behind `impl`.
pub use goxl_codec::dependencies;
