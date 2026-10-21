#![deny(rustdoc::broken_intra_doc_links)]

//! Converts between SDF Json documents and an sdfcore main.
//!
//! [`from_sdfj_file()`] loads an [`sdfj::SdfjFile`] into an
//! [`SdfMain`](sdfcore::SdfMain), and [`to_sdfj_file()`] encodes one back.
//! Each entry keeps its place in its table, so a document reads back as the
//! same state. The `codec` module, behind the default `codec` feature, goes
//! straight to and from `.sdfj` / `.sdfjz` bytes and takes the codec's
//! dependencies. `sdfj_codec::DependenciesImpl` supplies them.

// Public API

mod error;
mod from_sdfj_file;
mod result;
mod to_sdfj_file;

pub use error::*;
pub use from_sdfj_file::*;
pub use result::*;
pub use to_sdfj_file::*;

// Optional API

#[cfg(feature = "codec")]
pub mod codec;

// Test support

#[cfg(test)]
mod test;

#[cfg(test)]
pub(crate) use test::*;
