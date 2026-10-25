#![deny(rustdoc::broken_intra_doc_links)]

//! Reads and writes SDF Json `.sdfj` / `.sdfjz` documents over the data types
//! the `sdfj` crate defines. The JSON coding, deflate, and font lookup and
//! decoding come in through the [`dependencies`] traits.

// Public API

pub mod dependencies;

mod error;
mod from_sdfj_file_bytes;
mod from_sdfj_or_sdfjz_file_bytes;
mod from_sdfjz_file_bytes;
mod load_sdfj_fonts;
mod result;
mod sdfj_font_data;
mod sdfj_font_file;
mod sdfj_glyph;
mod sdfj_glyph_contour;
mod sdfj_glyph_segment;
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
pub use load_sdfj_fonts::*;
pub use result::*;
pub use sdfj_font_data::*;
pub use sdfj_font_file::*;
pub use sdfj_glyph::*;
pub use sdfj_glyph_contour::*;
pub use sdfj_glyph_segment::*;
pub use to_sdfj_file_bytes::*;
pub use to_sdfj_pretty_file_bytes::*;
pub use to_sdfjz_file_bytes::*;
pub use unwrap_sdfjz::*;
pub use wrap_sdfjz::*;

// Test support

#[cfg(all(test, feature = "impl"))]
mod test;

#[cfg(all(test, feature = "impl"))]
pub(crate) use test::*;
