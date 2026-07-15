use crate::TreeGridSwatch;
use serde_json::Value;

/// One value in a node's data series.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridValue {
    /// The pre-rendered display text.
    pub text: String,

    /// The native JSON form, stored separately from `text` because
    /// neither derives from the other: policy-rounded text may pair
    /// with a full-fidelity number. A value without one falls back to
    /// a JSON string of `text` in the JSON layouts.
    pub json: Option<Value>,

    /// The color block for swatch-rendering cell formats.
    pub swatch: Option<TreeGridSwatch>,
}

impl TreeGridValue {
    /// Creates a text-only value: no JSON form, no swatch.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            json: None,
            swatch: None,
        }
    }

    /// Sets the native JSON form.
    pub fn with_json(mut self, json: Value) -> Self {
        self.json = Some(json);
        self
    }

    /// Sets the swatch.
    pub fn with_swatch(mut self, swatch: TreeGridSwatch) -> Self {
        self.swatch = Some(swatch);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridSwatch, TreeGridValue};
    use serde_json::json;

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(value.text, "{node: 0}");
        assert_eq!(value.json, None);
        assert_eq!(value.swatch, None);
    }

    #[test]
    fn the_escape_hatch_sets_divergent_json_and_swatch() {
        let value = TreeGridValue::new("1.00")
            .with_json(json!(1.0))
            .with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(value.text, "1.00");
        assert_eq!(value.json, Some(json!(1.0)));
        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(255)));
    }
}
