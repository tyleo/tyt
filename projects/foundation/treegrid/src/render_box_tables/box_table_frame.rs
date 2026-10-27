use crate::{
    render::{Cell, TableFrame},
    render_box_tables,
};

/// The box frame: bare heading lines, box-glyph tables, and cells with
/// their newlines flattened.
pub struct BoxTableFrame;

impl TableFrame for BoxTableFrame {
    fn heading(&self, _depth: usize, text: &str) -> String {
        text.to_owned()
    }

    fn table(&self, headers: &[Cell], rows: &[Vec<Cell>]) -> String {
        let mut table = render_box_tables::box_table(headers, rows);
        // box_table ends its last line with `\n`; the block join re-adds
        // it.
        table.pop();
        table
    }

    /// A bare visual's opaque bytes pass verbatim.
    fn cell(&self, cell: Cell) -> Cell {
        if cell.bare_visual {
            return cell;
        }
        Cell {
            rendered: render_box_tables::box_cell(&cell.rendered),
            width: cell.width,
            bare_visual: false,
        }
    }
}
