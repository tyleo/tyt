use crate::{
    VoxValue,
    ext::{Error, Result, json::vox_value_from_json_value},
};
use serde::Serialize;

/// Encodes a format ext as the entry value it persists under its key: the
/// write half behind a [`VoxExtEntryCodec`](crate::ext::VoxExtEntryCodec)
/// impl. The ext serializes through serde_json, so its serde attributes
/// shape the value.
pub fn vox_value_from_ext<T: Serialize>(ext: &T) -> Result<VoxValue> {
    let value = serde_json::to_value(ext).map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(vox_value_from_json_value(value))
}
