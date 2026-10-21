//! Reads and writes `.sdfj` / `.sdfjz` document bytes, gated behind the
//! `codec` feature.

// Public API

mod from_sdfj_bytes;
mod to_sdfj_bytes;
mod to_sdfj_pretty_bytes;
mod to_sdfjz_bytes;

pub use from_sdfj_bytes::*;
pub use to_sdfj_bytes::*;
pub use to_sdfj_pretty_bytes::*;
pub use to_sdfjz_bytes::*;

// Re-exported so a caller can name the dependencies the functions here take and
// bind them through the codec's impl behind `impl`.
pub use sdfj_codec::dependencies;
