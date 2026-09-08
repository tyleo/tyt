//! The ext protocol: the ext a [`VoxMain`](crate::VoxMain) carries and its
//! untyped [`VoxMap`](crate::VoxMap) form in a document's ext block.
//!
//! [`VoxExt`] is the state level: the whole block a state persists, plus the
//! hooks the state fires when a listing moves so an ext aligned by listing
//! index can follow. [`VoxExtEntryCodec`] is the format level: one entry
//! under the format's vendor key, with [`encode_entry`] and [`decode_entry`]
//! moving between the entry and a one-entry block. [`VoxExtBlockCodec`]
//! builds a state's ext from a loaded block. [`CompositeVoxExt`] carries a
//! block entry by entry, each decoded into its format's ext or kept
//! verbatim. The [`json`] module, gated behind the `json` feature, holds the
//! serde transcode behind a format's entry codec.

#[cfg(feature = "json")]
pub mod json;

mod composite_vox_ext;
mod decode_entry;
mod encode_entry;
mod error;
mod follow_listing;
mod result;
mod vox_ext;
mod vox_ext_block_codec;
mod vox_ext_entry_codec;

pub use composite_vox_ext::*;
pub use decode_entry::*;
pub use encode_entry::*;
pub use error::*;
pub use follow_listing::*;
pub use result::*;
pub use vox_ext::*;
pub use vox_ext_block_codec::*;
pub use vox_ext_entry_codec::*;
