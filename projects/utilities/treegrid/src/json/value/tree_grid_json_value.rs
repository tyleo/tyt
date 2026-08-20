use crate::{TreeGridSwatch, TreeGridValue};
use serde_json::{Value, json};

/// One value in a node's data series, paired with its native JSON
/// form for the JSON layouts.
///
/// Rendered by
/// [`TreeGridJsonValueCells`](crate::TreeGridJsonValueCells): the
/// text layouts see the inner value, the JSON layouts the paired
/// form.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridJsonValue {
    /// The text-layout value.
    pub value: TreeGridValue,

    /// The native JSON form, stored separately from the text because
    /// neither derives from the other: policy-rounded text may pair
    /// with a full-fidelity number. A value without one falls back to
    /// a JSON string of its text in the JSON layouts.
    pub json: Option<Value>,
}

impl TreeGridJsonValue {
    /// Creates a text-only value: no JSON form, no swatch.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            value: TreeGridValue::new(text),
            json: None,
        }
    }

    /// Creates an integer value: [`TreeGridValue::int`] paired with
    /// integer JSON.
    pub fn int(value: i64) -> Self {
        Self {
            value: TreeGridValue::int(value),
            json: Some(json!(value)),
        }
    }

    /// Creates a float value: [`TreeGridValue::float`] paired with a
    /// JSON number that collapses like the text, an integer when the
    /// value is integral.
    pub fn float(value: f64) -> Self {
        Self {
            value: TreeGridValue::float(value),
            json: Some(number_json(value)),
        }
    }

    /// Creates a unit-interval value: [`TreeGridValue::unorm`] paired
    /// with a JSON number like [`float`](Self::float).
    pub fn unorm(value: f64) -> Self {
        Self {
            value: TreeGridValue::unorm(value),
            json: Some(number_json(value)),
        }
    }

    /// Creates an 8-bit unsigned-normalized value:
    /// [`TreeGridValue::unorm8`] paired with integer JSON.
    pub fn unorm8(value: u8) -> Self {
        Self {
            value: TreeGridValue::unorm8(value),
            json: Some(json!(value)),
        }
    }

    /// Creates a boolean value: [`TreeGridValue::bool`] paired with
    /// JSON bool.
    pub fn bool(value: bool) -> Self {
        Self {
            value: TreeGridValue::bool(value),
            json: Some(Value::Bool(value)),
        }
    }

    /// Creates a JSON value: its compact rendering as text and the
    /// value itself as the native form, no swatch.
    pub fn json(value: Value) -> Self {
        Self {
            value: TreeGridValue::new(json_text(&value)),
            json: Some(value),
        }
    }

    /// Creates an 8-bit sRGB color value: [`TreeGridValue::srgb8`]
    /// paired with the hex string as JSON.
    pub fn srgb8(rgb: [u8; 3]) -> Self {
        Self::hex_json(TreeGridValue::srgb8(rgb))
    }

    /// Creates an 8-bit sRGB color value with alpha:
    /// [`TreeGridValue::srgba8`] paired with the hex string as JSON.
    pub fn srgba8(rgba: [u8; 4]) -> Self {
        Self::hex_json(TreeGridValue::srgba8(rgba))
    }

    /// Sets the native JSON form.
    pub fn with_json(mut self, json: Value) -> Self {
        self.json = Some(json);
        self
    }

    /// Sets the swatch.
    pub fn with_swatch(mut self, swatch: TreeGridSwatch) -> Self {
        self.value = self.value.with_swatch(swatch);
        self
    }

    /// Pairs a hex-texted color value with its text as the JSON form.
    fn hex_json(value: TreeGridValue) -> Self {
        Self {
            json: Some(Value::String(value.text.clone())),
            value,
        }
    }
}

/// A number as JSON: an integer when it is integral and fits `i64`,
/// else a float, so it reads as it does in the text layouts. JSON has
/// no infinity and serde_json writes one as `null`; an infinite value
/// carries its `Display` text instead, the same text the cell shows.
pub(crate) fn number_json(value: f64) -> Value {
    if value.is_infinite() {
        Value::String(value.to_string())
    } else if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
        json!(value as i64)
    } else {
        json!(value)
    }
}

/// A JSON value as its compact text.
fn json_text(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridJsonValue, TreeGridSwatch, TreeGridValue};
    use serde_json::json;

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridJsonValue::new("{node: 0}");

        assert_eq!(value.value.text, "{node: 0}");
        assert_eq!(value.json, None);
        assert_eq!(value.value.swatch, None);
    }

    #[test]
    fn int_pairs_integer_json() {
        let value = TreeGridJsonValue::int(-3);

        assert_eq!(value.value, TreeGridValue::int(-3));
        assert_eq!(value.json, Some(json!(-3)));
    }

    #[test]
    fn float_collapses_an_integral_value_to_a_json_integer() {
        assert_eq!(TreeGridJsonValue::float(1.0).json, Some(json!(1)));
        assert_eq!(TreeGridJsonValue::float(0.2).json, Some(json!(0.2)));
    }

    #[test]
    fn an_infinite_float_carries_its_text_rather_than_null() {
        // serde_json writes a non-finite number as `null`, which would drop
        // the value the cell shows.
        let positive = TreeGridJsonValue::float(f64::INFINITY);
        let negative = TreeGridJsonValue::float(f64::NEG_INFINITY);

        assert_eq!(positive.json, Some(json!("inf")));
        assert_eq!(positive.value.text, "inf");
        assert_eq!(negative.json, Some(json!("-inf")));
        assert_eq!(negative.value.text, "-inf");
    }

    #[test]
    fn unorm_delegates_text_and_swatch_and_pairs_a_number() {
        let value = TreeGridJsonValue::unorm(0.5);

        assert_eq!(value.value, TreeGridValue::unorm(0.5));
        assert_eq!(value.json, Some(json!(0.5)));
    }

    #[test]
    fn unorm8_pairs_integer_json() {
        let value = TreeGridJsonValue::unorm8(128);

        assert_eq!(value.value, TreeGridValue::unorm8(128));
        assert_eq!(value.json, Some(json!(128)));
    }

    #[test]
    fn bool_pairs_json_bool() {
        let value = TreeGridJsonValue::bool(true);

        assert_eq!(value.value, TreeGridValue::bool(true));
        assert_eq!(value.json, Some(json!(true)));
    }

    #[test]
    fn json_renders_compact_text_and_keeps_the_value() {
        let value = TreeGridJsonValue::json(json!({"node": 0}));

        assert_eq!(value.value.text, "{\"node\":0}");
        assert_eq!(value.json, Some(json!({"node": 0})));
        assert_eq!(value.value.swatch, None);
    }

    #[test]
    fn srgb8_pairs_the_hex_string() {
        let value = TreeGridJsonValue::srgb8([255, 0, 128]);

        assert_eq!(value.value, TreeGridValue::srgb8([255, 0, 128]));
        assert_eq!(value.json, Some(json!("#FF0080")));
    }

    #[test]
    fn srgba8_pairs_the_hex_string() {
        let value = TreeGridJsonValue::srgba8([255, 0, 128, 64]);

        assert_eq!(value.value, TreeGridValue::srgba8([255, 0, 128, 64]));
        assert_eq!(value.json, Some(json!("#FF008040")));
    }

    #[test]
    fn with_json_sets_a_divergent_json_form() {
        let value = TreeGridJsonValue::new("1.00").with_json(json!(1.0));

        assert_eq!(value.value.text, "1.00");
        assert_eq!(value.json, Some(json!(1.0)));
    }

    #[test]
    fn with_swatch_sets_the_inner_swatch() {
        let value = TreeGridJsonValue::new("1.00").with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(value.value.swatch, Some(TreeGridSwatch::Gray(255)));
    }
}
