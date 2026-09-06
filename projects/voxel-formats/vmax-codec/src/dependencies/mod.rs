//! The dependencies the codec takes: LZFSE, the plist and scene JSON
//! codings, and PNG, injected so the crate carries none of those libraries.
//! [`DependenciesImpl`], behind the `impl` feature, binds them over `lzfse`,
//! `plist`, `png`, and `serde_json`.

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

#[cfg(feature = "impl")]
mod dependencies_impl;

#[cfg(feature = "impl")]
pub use dependencies_impl::*;
