//! The dependencies the codec takes: the JSON coding and deflate, injected
//! so the crate carries no JSON or compression library.
//! [`DependenciesImpl`], behind the `impl` feature, binds them and voxj's, so
//! one value serves the whole family.

// Public API

mod decode_voxj_json;
mod deflate;
mod encode_voxj_json;
mod inflate;

pub use decode_voxj_json::*;
pub use deflate::*;
pub use encode_voxj_json::*;
pub use inflate::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
