//! The serde transcode behind a format's
//! [`VoxExtEntryCodec`](crate::ext::VoxExtEntryCodec) impl, gated behind the
//! `json` feature. [`vox_value_from_ext`] and [`ext_from_vox_value`]
//! serialize an ext through serde_json, converting between the JSON and
//! voxcore value trees.

mod ext_from_vox_value;
mod json_value_from_vox_value;
mod vox_value_from_ext;
mod vox_value_from_json_value;

pub use ext_from_vox_value::*;
pub use json_value_from_vox_value::*;
pub use vox_value_from_ext::*;
pub use vox_value_from_json_value::*;
