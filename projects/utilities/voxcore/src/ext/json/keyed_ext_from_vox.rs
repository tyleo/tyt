use crate::{
    VoxMap,
    ext::{Error, Result, json::json_value_from_vox_value},
};
use serde::de::DeserializeOwned;

/// Decodes the format ext held under `key` in a document ext block, the read
/// half behind each format's [`VoxExtCodec`](crate::ext::VoxExtCodec) impl.
/// A block without `key` belongs to another format and yields `None`; a
/// present entry that does not decode as `T` is an error.
pub fn keyed_ext_from_vox<T: DeserializeOwned>(key: &str, ext: &VoxMap) -> Result<Option<T>> {
    let Some((_, value)) = ext.0.iter().find(|(name, _)| name == key) else {
        return Ok(None);
    };
    let value = json_value_from_vox_value(value)?;
    Ok(Some(
        T::deserialize(value).map_err(|error| Error::Invalid(error.to_string()))?,
    ))
}

#[cfg(test)]
mod tests {
    use crate::ext::json::{keyed_ext_from_vox, keyed_vox_ext};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, PartialEq, Serialize)]
    struct Ext {
        count: u32,
        scale: f64,
        name: String,
        tags: Vec<i32>,
    }

    fn ext() -> Ext {
        Ext {
            count: 7,
            scale: 1.5,
            name: "x".to_owned(),
            tags: vec![-1, 2],
        }
    }

    /// The keyed transcode round-trips a typed ext, keeping integer fields
    /// readable through the f64 value tree.
    #[test]
    fn round_trips_a_typed_ext() {
        let block = keyed_vox_ext("test", &ext()).unwrap();
        assert_eq!(keyed_ext_from_vox("test", &block).unwrap(), Some(ext()));
    }

    /// A block held under another format's key yields `None`.
    #[test]
    fn a_foreign_key_yields_none() {
        let block = keyed_vox_ext("other", &ext()).unwrap();
        assert_eq!(keyed_ext_from_vox::<Ext>("test", &block).unwrap(), None);
    }
}
