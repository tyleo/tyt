//! The dependencies the codec takes: zlib compression and decompression,
//! injected so the crate carries no compression library.
//! [`DependenciesImpl`], behind the `impl` feature, binds them over `flate2`.

mod compress_zlib;
mod decompress_zlib;

pub use compress_zlib::*;
pub use decompress_zlib::*;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
