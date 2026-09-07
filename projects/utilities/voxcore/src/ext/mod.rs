//! The ext protocol: how a typed state ext maps to and from the untyped
//! [`VoxMap`](crate::VoxMap) form a document's ext block carries.
//!
//! A persisted ext block is a value tree holding one entry per format, each
//! under that format's vendor key. A format's ext type implements
//! [`VoxExtCodec`] to encode itself under its key and to find itself in a
//! block again. The whole ext a [`VoxMain`](crate::VoxMain) carries, which
//! may be nothing, takes part in loads and writes through
//! [`VoxExtBlockCodec`]. The [`json`] module, gated behind the `json`
//! feature, holds the serde transcode behind each codec impl.

#[cfg(feature = "json")]
pub mod json;

mod error;
mod result;
mod vox_ext_block_codec;
mod vox_ext_codec;

pub use error::*;
pub use result::*;
pub use vox_ext_block_codec::*;
pub use vox_ext_codec::*;
