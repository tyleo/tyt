use crate::{Error, Result};
use serde::{Serialize, de::DeserializeOwned};
use voxj::VoxjMap;

/// Decodes the format ext held under `key` in a document `ext` block, the
/// read half behind each format's [`VoxjExtCodec`](crate::VoxjExtCodec) impl.
/// A block without `key` belongs to another format and yields `None`; a
/// present entry that does not decode as `T` is an error.
pub fn keyed_ext_from_voxj<T: DeserializeOwned>(key: &str, ext: &VoxjMap) -> Result<Option<T>> {
    let Some((_, value)) = ext.0.iter().find(|(name, _)| name == key) else {
        return Ok(None);
    };
    let value = value
        .serialize(serde_json::value::Serializer)
        .map_err(Error::invalid)?;
    Ok(Some(T::deserialize(value).map_err(Error::invalid)?))
}
