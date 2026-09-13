use gltf::json::Extras;
use serde_json::value::RawValue;

/// A JSON value as an object's `extras`, `None` for none.
pub fn extras_from_value(value: &Option<serde_json::Value>) -> Extras {
    value.as_ref().map(|value| {
        RawValue::from_string(value.to_string()).expect("a serialized value is valid JSON")
    })
}
