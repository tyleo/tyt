use crate::{Error, Result, ext::vox_value_from_json_value};
use serde::Serialize;
use voxcore::VoxValue;

/// Encodes a format's ext as the entry value it persists, the write half of
/// the transcode. The ext serializes through serde_json, so its serde
/// attributes shape the value.
pub fn vox_value_from_ext<T: Serialize>(ext: &T) -> Result<VoxValue> {
    let value = serde_json::to_value(ext).map_err(|error| Error::Ext(error.to_string()))?;
    Ok(vox_value_from_json_value(value))
}
