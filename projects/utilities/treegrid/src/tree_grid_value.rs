use crate::TreeGridSwatch;
#[cfg(feature = "json")]
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
    #[cfg(feature = "json")]
    pub json: Option<Value>,

    /// The color block for swatch-rendering cell formats.
    pub swatch: Option<TreeGridSwatch>,
}

impl TreeGridValue {
    /// Creates a text-only value: no JSON form, no swatch.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            #[cfg(feature = "json")]
            json: None,
            swatch: None,
        }
    }

    /// Sets the native JSON form.
    #[cfg(feature = "json")]
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
    #[cfg(feature = "json")]
    use serde_json::json;

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(value.text, "{node: 0}");
        #[cfg(feature = "json")]
        assert_eq!(value.json, None);
        assert_eq!(value.swatch, None);
    }

    #[test]
    fn with_swatch_sets_the_swatch() {
        let value = TreeGridValue::new("1.00").with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(255)));
    }

    #[cfg(feature = "json")]
    #[test]
    fn with_json_sets_a_divergent_json_form() {
        let value = TreeGridValue::new("1.00").with_json(json!(1.0));

        assert_eq!(value.text, "1.00");
        assert_eq!(value.json, Some(json!(1.0)));
    }
}
