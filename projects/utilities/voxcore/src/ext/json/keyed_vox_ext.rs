use crate::{
    VoxMap,
    ext::{Error, Result, json::vox_value_from_json_value},
};
use serde::Serialize;

/// Encodes a format ext into the block form held under `key`, the write half
/// behind each format's [`VoxExtCodec`](crate::ext::VoxExtCodec) impl. The
/// ext serializes through serde_json, so its serde attributes shape the
/// block.
pub fn keyed_vox_ext<T: Serialize>(key: &str, ext: &T) -> Result<VoxMap> {
    let value = serde_json::to_value(ext).map_err(|error| Error::Invalid(error.to_string()))?;
    Ok(VoxMap(vec![(
        key.to_owned(),
        vox_value_from_json_value(value),
    )]))
}
