// Public API

mod dependencies;
mod dependencies_impl;
mod deserialize_prefs;
mod dir_prefs;
mod load;
mod prefs;
mod prefs_paths;
mod read_section;
mod resolve;
mod serialize_prefs;
mod write_section;

pub use dependencies::*;
pub use dependencies_impl::*;
pub use deserialize_prefs::*;
pub use dir_prefs::*;
pub use load::*;
pub use prefs::*;
pub use prefs_paths::*;
pub use read_section::*;
pub use resolve::*;
pub use serialize_prefs::*;
pub use write_section::*;

// Optional API

#[cfg(feature = "json-codec")]
mod json_codec;

#[cfg(feature = "json-codec")]
pub use json_codec::*;

#[cfg(feature = "jsonc-codec")]
mod jsonc_codec;

#[cfg(feature = "jsonc-codec")]
pub use jsonc_codec::*;
