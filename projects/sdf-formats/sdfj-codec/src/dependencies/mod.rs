//! The dependencies the codec takes: the JSON coding, deflate, and font lookup
//! and decoding, injected so the crate carries no JSON, compression, or font
//! library.
//! [`DependenciesImpl`], behind the `impl` feature, binds them.

// Public API

mod decode_font;
mod decode_sdfj_json;
mod deflate;
mod encode_sdfj_json;
mod find_installed_font;
mod inflate;
mod read_font_file;

pub use decode_font::*;
pub use decode_sdfj_json::*;
pub use deflate::*;
pub use encode_sdfj_json::*;
pub use find_installed_font::*;
pub use inflate::*;
pub use read_font_file::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
