use crate::{VoxMap, VoxValue};
use serde_json::Value;

/// Converts a serde_json [`Value`] into a [`VoxValue`], recursing into arrays
/// and objects. The write half of the typed-ext transcode: a format ext
/// serializes to JSON values, which land in the state as voxcore values.
pub fn vox_value_from_json_value(value: Value) -> VoxValue {
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
        Value::Object(object) => VoxValue::Object(VoxMap(
            object
                .into_iter()
                .map(|(key, value)| (key, vox_value_from_json_value(value)))
                .collect(),
        )),
    }
}
