use crate::{Error, Result, ext::json_value_from_vox_value};
use serde::de::DeserializeOwned;
use voxcore::VoxValue;

/// Decodes a format's ext from its entry value, the read half of the
/// transcode. A value that does not decode as `T` is an error.
pub fn ext_from_vox_value<T: DeserializeOwned>(value: &VoxValue) -> Result<T> {
    let value = json_value_from_vox_value(value)?;
    T::deserialize(value).map_err(|error| Error::Ext(error.to_string()))
}

#[cfg(test)]
mod tests {
    use crate::ext::{ext_from_vox_value, vox_value_from_ext};
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

    /// The transcode round-trips a typed ext, keeping integer fields
    /// readable through the f64 value tree.
    #[test]
    fn round_trips_a_typed_ext() {
        let value = vox_value_from_ext(&ext()).unwrap();

        assert_eq!(ext_from_vox_value::<Ext>(&value).unwrap(), ext());
    }
}
