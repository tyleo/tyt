use crate::{Error, Result, vox_value_from_json_value};
use serde::Serialize;
use voxcore::VoxMap;

/// Encodes a format ext into the block form held under `key`, the write half
/// behind each format's [`VoxjExtCodec`](crate::VoxjExtCodec) impl. The ext
/// serializes through serde_json, so its serde attributes shape the block.
pub fn keyed_voxj_ext<T: Serialize>(key: &str, ext: &T) -> Result<VoxMap> {
    let value = serde_json::to_value(ext).map_err(Error::invalid)?;
    Ok(VoxMap(vec![(
        key.to_owned(),
        vox_value_from_json_value(value),
    )]))
}
