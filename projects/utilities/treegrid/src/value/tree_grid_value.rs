use crate::TreeGridSwatch;

/// One value in a node's data series.
///
/// Its JSON form is always a JSON string of its text; behind the
/// `json` feature, `TreeGridJsonValue` pairs a value with a native
/// JSON form for pairs that genuinely diverge.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeGridValue {
    /// The pre-rendered display text.
    pub text: String,

    /// The color block the default policy renders as this value's
    /// visual.
    pub swatch: Option<TreeGridSwatch>,
}

impl TreeGridValue {
    /// Creates a text-only value: no swatch.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            swatch: None,
        }
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

    #[test]
    fn new_builds_a_text_only_value() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(value.text, "{node: 0}");
        assert_eq!(value.swatch, None);
    }

    #[test]
    fn with_swatch_sets_the_swatch() {
        let value = TreeGridValue::new("1.00").with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(value.swatch, Some(TreeGridSwatch::Gray(255)));
    }
}
