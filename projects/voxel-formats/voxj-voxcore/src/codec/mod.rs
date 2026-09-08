//! Reads and writes `.voxj` / `.voxjz` document bytes, gated behind the
//! `codec` feature.

mod check_voxj_bytes;
mod from_voxj_bytes;
mod to_voxj_bytes;
mod to_voxj_pretty_bytes;
mod to_voxjz_bytes;
mod voxj_version_from_bytes;

pub use check_voxj_bytes::*;
pub use from_voxj_bytes::*;
pub use to_voxj_bytes::*;
pub use to_voxj_pretty_bytes::*;
pub use to_voxjz_bytes::*;
pub use voxj_version_from_bytes::*;

// Re-exported so a caller can name the dependencies the functions here take
// and bind them through the codec's impl behind `impl`.
pub use voxj_codec::dependencies;
