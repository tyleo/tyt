use crate::{
    VoxMap, VoxValue,
    ext::{Error, Result},
};
use serde_json::{Map, Number, Value};

/// Converts a [`VoxValue`] into the serde_json [`Value`] a format ext
/// deserializes from, recursing into arrays and objects. An integral number
/// becomes a JSON integer, so an ext's integer fields read back. Errors on a
/// non-finite number, which has no JSON form, and on a repeated object key.
pub fn json_value_from_vox_value(value: &VoxValue) -> Result<Value> {
    Ok(match value {
        VoxValue::Null => Value::Null,
        VoxValue::Bool(bool) => Value::Bool(*bool),
        VoxValue::Number(number) if !number.is_finite() => {
            return Err(Error::Invalid(format!("number {number} must be finite")));
        }
        // The bound is exclusive because `i64::MAX as f64` rounds up to
        // `2^63`, past what an i64 holds.
        VoxValue::Number(number)
            if number.fract() == 0.0 && *number >= i64::MIN as f64 && *number < i64::MAX as f64 =>
        {
            Value::Number(Number::from(*number as i64))
        }
        VoxValue::Number(number) => {
            Value::Number(Number::from_f64(*number).expect("a finite number has a json form"))
        }
        VoxValue::Text(text) => Value::String(text.clone()),
        VoxValue::Array(array) => Value::Array(
            array
                .iter()
                .map(json_value_from_vox_value)
                .collect::<Result<_>>()?,
        ),
        VoxValue::Object(VoxMap(entries)) => {
            let mut object = Map::with_capacity(entries.len());
            for (key, value) in entries {
                let value = json_value_from_vox_value(value)?;
                if object.insert(key.clone(), value).is_some() {
                    return Err(Error::Invalid(format!(
                        "json object key `{key}` must be unique"
                    )));
                }
            }
            Value::Object(object)
        }
    })
}
