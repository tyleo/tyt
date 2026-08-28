use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use voxj::{VoxjMap, VoxjValue};

/// Encodes a format ext into a document `ext` block holding it under `key`,
/// the write half behind each format's
/// [`VoxjExtCodec`](crate::VoxjExtCodec) impl. The ext serializes through
/// [`serde_json::Value`], so integral numbers stay integral in the document.
pub fn keyed_voxj_ext<T: Serialize>(key: &str, ext: &T) -> Result<VoxjMap> {
    let value = serde_json::to_value(ext).map_err(Error::invalid)?;
    let value = VoxjValue::deserialize(value).map_err(Error::invalid)?;
    Ok(VoxjMap(vec![(key.to_owned(), value)]))
}
