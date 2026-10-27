use crate::render::Cell;

/// One box-glyph table: a top edge, the header row, then the rows with a
/// rule under the header and between every pair of rows, and a bottom
/// edge, every column padded to its widest cell. A row shorter than the
/// headers leaves its trailing columns blank. Width is each cell's
/// declared visible width, so visual cells line up too.
pub fn box_table(headers: &[Cell], rows: &[Vec<Cell>]) -> String {
    let mut widths: Vec<usize> = headers.iter().map(|header| header.width).collect();
    for row in rows {
        for (column, cell) in row.iter().enumerate().take(widths.len()) {
            widths[column] = widths[column].max(cell.width);
        }
    }

    let mut output = edge(&widths, '┌', '┬', '┐');
    output.push_str(&table_row(headers, &widths));
    for row in rows {
        output.push_str(&edge(&widths, '├', '┼', '┤'));
        output.push_str(&table_row(row, &widths));
    }
    output.push_str(&edge(&widths, '└', '┴', '┘'));
    output
}

/// One horizontal edge: `left`, a `─` run two wider than each column
/// joined by `junction`, then `right`.
fn edge(widths: &[usize], left: char, junction: char, right: char) -> String {
    let runs: Vec<String> = widths.iter().map(|width| "─".repeat(width + 2)).collect();
    format!("{left}{}{right}\n", runs.join(&junction.to_string()))
}

/// One `│ ... │ ... │` row, each column padded to its width by the cell's
/// declared visible width, a missing trailing column left blank.
fn table_row(cells: &[Cell], widths: &[usize]) -> String {
    let mut line = String::from("│");
    for (column, width) in widths.iter().enumerate() {
        let (rendered, cell_width) = cells
            .get(column)
            .map_or(("", 0), |cell| (cell.rendered.as_str(), cell.width));
        line.push(' ');
        line.push_str(rendered);
        line.push_str(&" ".repeat(width.saturating_sub(cell_width)));
        line.push_str(" │");
    }
    line.push('\n');
    line
}

#[cfg(test)]
mod tests {
    use crate::{render::Cell, render_box_tables};

    #[test]
    fn aligns_columns_to_the_widest_cell() {
        let headers = [Cell::text("#"), Cell::text("name")];
        let rows = [
            vec![Cell::text("0"), Cell::text("energy-tank")],
            vec![Cell::text("1"), Cell::text("door")],
        ];

        assert_eq!(
            render_box_tables::box_table(&headers, &rows),
            "┌───┬─────────────┐\n\
             │ # │ name        │\n\
             ├───┼─────────────┤\n\
             │ 0 │ energy-tank │\n\
             ├───┼─────────────┤\n\
             │ 1 │ door        │\n\
             └───┴─────────────┘\n"
        );
    }

    #[test]
    fn a_short_row_leaves_trailing_columns_blank() {
        let headers = [Cell::text("#"), Cell::text("a"), Cell::text("b")];
        let rows = [vec![Cell::text("0"), Cell::text("x")]];

        assert_eq!(
            render_box_tables::box_table(&headers, &rows),
            "┌───┬───┬───┐\n\
             │ # │ a │ b │\n\
             ├───┼───┼───┤\n\
             │ 0 │ x │   │\n\
             └───┴───┴───┘\n"
        );
    }

    #[test]
    fn a_visual_cell_pads_by_its_declared_width() {
        let cell = Cell {
            rendered: "\x1b[48;2;255;0;0m  \x1b[0m".to_owned(),
            width: 2,
            bare_visual: true,
        };

        assert_eq!(
            render_box_tables::box_table(&[Cell::text("color")], &[vec![cell]]),
            "┌───────┐\n\
             │ color │\n\
             ├───────┤\n\
             │ \x1b[48;2;255;0;0m  \x1b[0m    │\n\
             └───────┘\n"
        );
    }
}
