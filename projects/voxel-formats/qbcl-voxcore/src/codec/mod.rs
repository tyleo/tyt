//! Reads and writes `.qb`, `.qbt`, and `.qbcl` file bytes, gated behind the
//! `codec` feature. The `_with_ext` pairs move the bytes through the typed
//! path.

mod from_qb_bytes;
mod from_qbcl_bytes;
mod from_qbt_bytes;
mod to_qb_bytes;
mod to_qbcl_bytes;
mod to_qbt_bytes;

pub use from_qb_bytes::*;
pub use from_qbcl_bytes::*;
pub use from_qbt_bytes::*;
pub use to_qb_bytes::*;
pub use to_qbcl_bytes::*;
pub use to_qbt_bytes::*;

mod from_qb_bytes_with_ext;

mod from_qbcl_bytes_with_ext;

mod from_qbt_bytes_with_ext;

mod to_qb_bytes_with_ext;

mod to_qbcl_bytes_with_ext;

mod to_qbt_bytes_with_ext;

pub use from_qb_bytes_with_ext::*;

pub use from_qbcl_bytes_with_ext::*;

pub use from_qbt_bytes_with_ext::*;

pub use to_qb_bytes_with_ext::*;

pub use to_qbcl_bytes_with_ext::*;

pub use to_qbt_bytes_with_ext::*;

// Re-exported so a caller can name the dependencies the functions here take
// and bind them through the codec's impl behind `impl`.
pub use qbcl_codec::dependencies;
