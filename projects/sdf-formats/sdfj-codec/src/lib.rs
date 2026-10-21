#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes SDF Json `.sdfj` / `.sdfjz` documents over the data types
//! the `sdfj` crate defines. The JSON coding and deflate come in through the
//! [`dependencies`] traits.

// Public API

pub mod dependencies;

mod error;
mod from_sdfj_file_bytes;
mod from_sdfj_or_sdfjz_file_bytes;
mod from_sdfjz_file_bytes;
mod result;
mod to_sdfj_file_bytes;
mod to_sdfj_pretty_file_bytes;
mod to_sdfjz_file_bytes;
mod unwrap_sdfjz;
mod wrap_sdfjz;

pub use dependencies::*;
pub use error::*;
pub use from_sdfj_file_bytes::*;
pub use from_sdfj_or_sdfjz_file_bytes::*;
pub use from_sdfjz_file_bytes::*;
pub use result::*;
pub use to_sdfj_file_bytes::*;
pub use to_sdfj_pretty_file_bytes::*;
pub use to_sdfjz_file_bytes::*;
pub use unwrap_sdfjz::*;
pub use wrap_sdfjz::*;
