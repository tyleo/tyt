use serde_json::{Value, json};

/// A number as JSON: an integer when it is integral and fits `i64`,
/// else a float, so it reads as it does in the text layouts. JSON has
/// no infinity and serde_json writes one as `null`; an infinite value
/// carries its `Display` text instead, the same text the cell shows.
pub fn number_json(value: f64) -> Value {
    if value.is_infinite() {
        Value::String(value.to_string())
    } else if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
        json!(value as i64)
    } else {
        json!(value)
    }
}
