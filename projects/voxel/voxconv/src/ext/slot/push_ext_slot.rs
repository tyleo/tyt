use crate::{
    Error, Result,
    ext::{VoxconvExt, push_slot},
};
use serde::Serialize;
use serde_json::Value;
use voxcore::{VoxMap, VoxMapEntry, VoxValue};

/// Appends the slot under `key` a boxed ext encodes to when it holds an
/// `E`, and reports whether it did. Errors when the block already holds the
/// key.
pub fn push_ext_slot<E: VoxconvExt + Serialize>(
    key: &str,
    ext: &dyn VoxconvExt,
    slots: &mut Vec<VoxMapEntry>,
) -> Result<bool> {
    let Some(ext) = ext.downcast_ref::<E>() else {
        return Ok(false);
    };

    push_slot(
        slots,
        VoxMapEntry {
            key: key.to_owned(),
            value: vox_value_from_ext(ext)?,
        },
    )?;

    Ok(true)
}

/// Encodes a format's ext as the entry value it persists, the write half of
/// the transcode. The ext serializes through serde_json, so its serde
/// attributes shape the value.
fn vox_value_from_ext<T: Serialize>(ext: &T) -> Result<VoxValue> {
    let value = serde_json::to_value(ext).map_err(|error| Error::Ext(error.to_string()))?;
    Ok(vox_value_from_json_value(value))
}

/// Converts a serde_json [`Value`] tree into a [`VoxValue`] tree.
fn vox_value_from_json_value(value: Value) -> VoxValue {
    match value {
        Value::Null => VoxValue::Null,

        Value::Bool(bool) => VoxValue::Bool(bool),

        Value::Number(number) => VoxValue::Number(
            number
                .as_f64()
                .expect("a json number without arbitrary_precision reads as f64"),
        ),

        Value::String(text) => VoxValue::Text(text),

        Value::Array(array) => {
            VoxValue::Array(array.into_iter().map(vox_value_from_json_value).collect())
        }

        Value::Object(object) => VoxValue::Object(VoxMap::new(
            object
                .into_iter()
                .map(|(key, value)| VoxMapEntry {
                    key,
                    value: vox_value_from_json_value(value),
                })
                .collect(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use crate::ext::{ext_from_vox_value, slot::push_ext_slot::vox_value_from_ext};
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
