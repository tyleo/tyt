use crate::{
    TreeGridCellFormat, TreeGridCells,
    render::{self, Cell},
};

impl Cell {
    /// Renders one value's cell under `format`, the node's when set, else the
    /// policy's for the value; a value without a visual renders its text under
    /// every format.
    pub(crate) fn render<C: TreeGridCells>(
        cells: &C,
        format: Option<TreeGridCellFormat>,
        value: &C::Value,
    ) -> Self {
        let format = format.unwrap_or_else(|| cells.format(value));
        let visual = match format {
            TreeGridCellFormat::Visual | TreeGridCellFormat::VisualText => cells.visual(value),
            TreeGridCellFormat::Text => None,
        };
        match (format, visual) {
            (TreeGridCellFormat::Visual, Some(visual)) => Self {
                rendered: visual.rendered,
                width: visual.width,
                bare_visual: true,
            },
            (TreeGridCellFormat::VisualText, Some(visual)) => {
                let text = cells.text(value);
                let width = visual.width + 1 + render::visible_width(&text);
                Self {
                    rendered: format!("{} {}", visual.rendered, text),
                    width,
                    bare_visual: false,
                }
            }
            _ => Self::text(cells.text(value).into_owned()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        TreeGridCellFormat, TreeGridSwatch, TreeGridValue, TreeGridValueCells, render::Cell,
    };

    fn color() -> TreeGridValue {
        TreeGridValue::new("#FF0000FF").with_swatch(TreeGridSwatch::Color([255, 0, 0]))
    }

    fn gray() -> TreeGridValue {
        TreeGridValue::new("0.5").with_swatch(TreeGridSwatch::Gray(128))
    }

    #[test]
    fn visual_format_renders_the_bare_visual() {
        let cell = Cell::render(
            &TreeGridValueCells,
            Some(TreeGridCellFormat::Visual),
            &color(),
        );

        assert_eq!(cell.rendered, "\x1b[48;2;255;0;0m  \x1b[0m");
        assert_eq!(cell.width, 2);
    }

    #[test]
    fn visual_text_format_renders_the_visual_beside_the_text() {
        let cell = Cell::render(
            &TreeGridValueCells,
            Some(TreeGridCellFormat::VisualText),
            &color(),
        );

        assert_eq!(cell.rendered, "\x1b[48;2;255;0;0m  \x1b[0m #FF0000FF");
        assert_eq!(cell.width, 12);
    }

    #[test]
    fn text_format_renders_the_text_alone() {
        let cell = Cell::render(
            &TreeGridValueCells,
            Some(TreeGridCellFormat::Text),
            &color(),
        );

        assert_eq!(cell.rendered, "#FF0000FF");
        assert_eq!(cell.width, 9);
    }

    #[test]
    fn a_value_without_a_visual_renders_text_under_every_format() {
        let value = TreeGridValue::new("true");
        for format in [
            TreeGridCellFormat::Visual,
            TreeGridCellFormat::VisualText,
            TreeGridCellFormat::Text,
        ] {
            let cell = Cell::render(&TreeGridValueCells, Some(format), &value);

            assert_eq!(cell.rendered, "true");
            assert_eq!(cell.width, 4);
        }
    }

    #[test]
    fn an_unset_format_defers_to_the_policy_per_value() {
        let decorated = Cell::render(&TreeGridValueCells, None, &color());
        let plain = Cell::render(&TreeGridValueCells, None, &gray());

        assert_eq!(decorated.rendered, "\x1b[48;2;255;0;0m  \x1b[0m #FF0000FF");
        assert_eq!(plain.rendered, "0.5");
    }
}
