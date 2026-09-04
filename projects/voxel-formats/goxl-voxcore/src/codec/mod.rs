//! Reads and writes `.gox` file bytes, gated behind the `codec` feature.

mod from_goxl_bytes;
mod to_goxl_bytes;

pub use from_goxl_bytes::*;
pub use to_goxl_bytes::*;
