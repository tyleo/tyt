use crate::{TreeGridCellFormat, TreeGridCells, TreeGridSwatch, TreeGridValue, TreeGridVisual};
use std::borrow::Cow;

/// The default cell policy, over pre-rendered
/// [`TreeGridValue`]s.
///
/// Text is the value's `text` and the visual renders the value's
/// swatch. A node with no explicit format shows a `Color`-swatched
/// value as its swatch beside its text and every other value as text
/// alone, so gray component swatches decorate only under an explicit
/// format.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TreeGridValueCells;

impl TreeGridCells for TreeGridValueCells {
    type Value = TreeGridValue;

    fn text<'a>(&self, value: &'a TreeGridValue) -> Cow<'a, str> {
        Cow::Borrowed(&value.text)
    }

    fn visual(&self, value: &TreeGridValue) -> Option<TreeGridVisual> {
        value.swatch.as_ref().map(TreeGridSwatch::render)
    }

    fn format(&self, value: &TreeGridValue) -> TreeGridCellFormat {
        match value.swatch {
            Some(TreeGridSwatch::Color(_)) => TreeGridCellFormat::VisualText,
            _ => TreeGridCellFormat::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridCellFormat, TreeGridCells, TreeGridSwatch, TreeGridValue, TreeGridValueCells,
    };

    #[test]
    fn text_borrows_the_value_text() {
        let value = TreeGridValue::new("#FF0000FF");

        assert_eq!(TreeGridValueCells.text(&value), "#FF0000FF");
    }

    #[test]
    fn the_visual_renders_the_swatch() {
        let value = TreeGridValue::new("#FF0000FF").with_swatch(TreeGridSwatch::Color([255, 0, 0]));

        assert_eq!(
            TreeGridValueCells.visual(&value),
            Some(TreeGridSwatch::Color([255, 0, 0]).render())
        );
    }

    #[test]
    fn a_swatchless_value_has_no_visual() {
        let value = TreeGridValue::new("1.00");

        assert_eq!(TreeGridValueCells.visual(&value), None);
    }

    #[test]
    fn a_color_swatch_defaults_to_visual_text_format() {
        let value = TreeGridValue::new("#FF0000FF").with_swatch(TreeGridSwatch::Color([255, 0, 0]));

        assert_eq!(
            TreeGridValueCells.format(&value),
            TreeGridCellFormat::VisualText
        );
    }

    #[test]
    fn a_gray_swatch_defaults_to_text_format() {
        let value = TreeGridValue::new("1.00").with_swatch(TreeGridSwatch::Gray(255));

        assert_eq!(TreeGridValueCells.format(&value), TreeGridCellFormat::Text);
    }

    #[test]
    fn a_swatchless_value_defaults_to_text_format() {
        let value = TreeGridValue::new("{node: 0}");

        assert_eq!(TreeGridValueCells.format(&value), TreeGridCellFormat::Text);
    }
}
