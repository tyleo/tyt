use crate::VMaxValue;
use serde::{Serialize, Serializer, ser::Error as SerError};

/// Serializes a `scene.json` field holding an untyped [`VMaxValue`]. A data
/// blob, a NaN, or an infinity errors because none reads back from JSON
/// unchanged.
pub fn json_value<S>(value: &Option<VMaxValue>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(value) = value {
        check_json_value(value).map_err(SerError::custom)?;
    }

    value.serialize(serializer)
}

/// Checks that everything in `value` has a JSON form.
fn check_json_value(value: &VMaxValue) -> Result<(), String> {
    match value {
        VMaxValue::Array(items) => items.iter().try_for_each(check_json_value),

        VMaxValue::Boolean(_) | VMaxValue::Integer(_) | VMaxValue::String(_) => Ok(()),

        VMaxValue::Data(_) => Err("a data blob has no JSON form".to_owned()),

        VMaxValue::Dictionary(entries) => entries.values().try_for_each(check_json_value),

        VMaxValue::Real(number) if !number.is_finite() => {
            Err(format!("number must be finite, not {number}"))
        }

        VMaxValue::Real(_) => Ok(()),
    }
}
