//! The dependencies the codec takes: PNG decoding and encoding, injected
//! so the crate carries no image library. [`DependenciesImpl`], behind the
//! `impl` feature, binds them over `png`.

mod decode_png;
mod encode_png;

pub use decode_png::*;
pub use encode_png::*;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
