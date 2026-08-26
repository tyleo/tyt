mod load;
pub use load::*;

mod dependencies;
mod dependencies_impl;
mod deserialize_prefs;
mod dir_prefs;
mod optional_dir_prefs;
mod prefs;
mod read_section;
mod serialize_prefs;
mod write_section;

pub use dependencies::*;
pub use dependencies_impl::*;
pub use deserialize_prefs::*;
pub use dir_prefs::*;
pub use optional_dir_prefs::*;
pub use prefs::*;
pub use read_section::*;
pub use serialize_prefs::*;
pub use write_section::*;

#[cfg(feature = "json-codec")]
mod json_codec;
#[cfg(feature = "json-codec")]
pub use json_codec::*;

#[cfg(feature = "jsonc-codec")]
mod jsonc_codec;
#[cfg(feature = "jsonc-codec")]
pub use jsonc_codec::*;
