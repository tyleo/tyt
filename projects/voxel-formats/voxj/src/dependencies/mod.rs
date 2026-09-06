//! The dependencies the document types take: base64 for the encoded
//! blocks and a cost the block-encoding search minimizes, injected so the
//! crate carries no base64 library. [`DependenciesImpl`], behind the `impl`
//! feature, binds them over `base64` and a deflate cost.

mod cost_voxj_object;
mod decode_base64;
mod encode_base64;

pub use cost_voxj_object::*;
pub use decode_base64::*;
pub use encode_base64::*;

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
