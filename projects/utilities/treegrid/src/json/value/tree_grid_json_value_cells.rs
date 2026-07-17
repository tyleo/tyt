use crate::{
    TreeGridCellFormat, TreeGridCells, TreeGridJsonCells, TreeGridJsonValue, TreeGridValueCells,
    TreeGridVisual,
};
use serde_json::Value;
use std::borrow::Cow;

/// The JSON-carrying cell policy, over [`TreeGridJsonValue`]s.
///
/// The text layouts render the inner
/// [`TreeGridValue`](crate::TreeGridValue) exactly as
/// [`TreeGridValueCells`] does; the JSON layouts emit the paired
/// native form, falling back to a JSON string of the text.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridJsonValueCells;

impl TreeGridCells for TreeGridJsonValueCells {
    type Value = TreeGridJsonValue;

    fn text<'a>(&self, value: &'a TreeGridJsonValue) -> Cow<'a, str> {
        TreeGridValueCells.text(&value.value)
    }

    fn visual(&self, value: &TreeGridJsonValue) -> Option<TreeGridVisual> {
        TreeGridValueCells.visual(&value.value)
    }

    fn format(&self, value: &TreeGridJsonValue) -> TreeGridCellFormat {
        TreeGridValueCells.format(&value.value)
    }
}

impl TreeGridJsonCells for TreeGridJsonValueCells {
    fn json(&self, value: &TreeGridJsonValue) -> Value {
        value
            .json
            .clone()
            .unwrap_or_else(|| Value::String(value.value.text.clone()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridCellFormat, TreeGridCells, TreeGridJsonCells, TreeGridJsonValue,
        TreeGridJsonValueCells, TreeGridSwatch,
    };
    use serde_json::{Value, json};

    #[test]
    fn the_text_layouts_delegate_to_the_default_policy() {
        let value = TreeGridJsonValue::new("#FF0000FF")
            .with_json(json!("#FF0000FF"))
            .with_swatch(TreeGridSwatch::Color([255, 0, 0]));

        assert_eq!(TreeGridJsonValueCells.text(&value), "#FF0000FF");
        assert_eq!(
            TreeGridJsonValueCells.visual(&value),
            Some(TreeGridSwatch::Color([255, 0, 0]).render())
        );
        assert_eq!(
            TreeGridJsonValueCells.format(&value),
            TreeGridCellFormat::VisualText
        );
    }

    #[test]
    fn json_passes_the_native_form_through() {
        let value = TreeGridJsonValue::new("1.00").with_json(json!(1.0));

        assert_eq!(TreeGridJsonValueCells.json(&value), json!(1.0));
    }

    #[test]
    fn json_falls_back_to_the_text_as_a_string() {
        let value = TreeGridJsonValue::new("{node: 0}");

        assert_eq!(
            TreeGridJsonValueCells.json(&value),
            Value::String("{node: 0}".to_owned())
        );
    }
}
