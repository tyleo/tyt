use crate::render::Cell;

impl Cell {
    /// The separator a node's cells join with: none when every cell is a bare
    /// visual, so they abut into a continuous strip, else one space.
    pub(crate) fn separator(cells: &[Cell]) -> &'static str {
        if cells.iter().all(|cell| cell.bare_visual) {
            ""
        } else {
            " "
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
    fn a_strip_of_bare_visuals_joins_with_no_separator() {
        let cells = [
            Cell::render(
                &TreeGridValueCells,
                Some(TreeGridCellFormat::Visual),
                &color(),
            ),
            Cell::render(
                &TreeGridValueCells,
                Some(TreeGridCellFormat::Visual),
                &gray(),
            ),
        ];

        assert_eq!(Cell::separator(&cells), "");
    }

    #[test]
    fn a_visual_less_cell_keeps_the_one_space_separator() {
        let cells = [
            Cell::render(
                &TreeGridValueCells,
                Some(TreeGridCellFormat::Visual),
                &color(),
            ),
            Cell::render(
                &TreeGridValueCells,
                Some(TreeGridCellFormat::Visual),
                &TreeGridValue::new("true"),
            ),
        ];

        assert_eq!(Cell::separator(&cells), " ");
    }

    #[test]
    fn a_decorated_cell_keeps_the_one_space_separator() {
        let cells = [Cell::render(&TreeGridValueCells, None, &color())];

        assert_eq!(Cell::separator(&cells), " ");
    }
}
