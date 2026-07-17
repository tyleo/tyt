use crate::{TreeGridSwatch, TreeGridValue};
use serde_json::Value;

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
}

#[cfg(test)]
mod tests {
    use crate::{TreeGridJsonValue, TreeGridSwatch};
    use serde_json::json;

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridJsonValue::new("{node: 0}");

        assert_eq!(value.value.text, "{node: 0}");
        assert_eq!(value.json, None);
        assert_eq!(value.value.swatch, None);
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
