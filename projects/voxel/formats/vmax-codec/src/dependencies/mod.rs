//! The dependencies the codec takes, injected so the crate carries no codec
//! library. [`DependenciesImpl`], behind the `impl` feature, binds them and
//! vmax's, so one value serves the whole family.

// Public API

mod compress_lzfse;
mod decode_png;
mod decode_vmax_plist;
mod decode_vmax_scene_json;
mod decompress_lzfse;
mod encode_png;
mod encode_vmax_plist;
mod encode_vmax_scene_json;

pub use compress_lzfse::*;
pub use decode_png::*;
pub use decode_vmax_plist::*;
pub use decode_vmax_scene_json::*;
pub use decompress_lzfse::*;
pub use encode_png::*;
pub use encode_vmax_plist::*;
pub use encode_vmax_scene_json::*;

// Optional API

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
