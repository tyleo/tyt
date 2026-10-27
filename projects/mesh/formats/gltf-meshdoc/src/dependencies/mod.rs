//! The dependencies the converters take: base64 decoding and encoding for
//! data URIs, injected so the crate carries no encoder. `DependenciesImpl`,
//! behind the `impl` feature, binds them.

// Public API

mod decode_base64;
mod encode_base64;

pub use decode_base64::*;
pub use encode_base64::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
