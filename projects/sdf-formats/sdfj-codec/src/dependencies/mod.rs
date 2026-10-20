//! The dependencies the codec takes: the JSON coding, injected so the crate
//! carries no JSON library. [`DependenciesImpl`], behind the `impl` feature,
//! binds them.

// Public API

mod decode_sdfj_json;
mod encode_sdfj_json;

pub use decode_sdfj_json::*;
pub use encode_sdfj_json::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
