use crate::{
    render::{self, Cell, TableFrame},
    render_md_tables,
};
use std::num::NonZeroU8;

/// The markdown frame: `#` headings from `level` down, pipe tables, and
/// cells with their pipes escaped.
pub struct MdTableFrame {
    /// The level of the shallowest heading.
    pub(crate) level: NonZeroU8,
}

impl TableFrame for MdTableFrame {
    fn heading(&self, depth: usize, text: &str) -> String {
        render::heading(self.level, depth, text)
    }

    fn table(&self, headers: &[Cell], rows: &[Vec<Cell>]) -> String {
        let mut table = render_md_tables::md_table(headers, rows);
        // md_table ends its last line with `\n`; the block join re-adds
        // it.
        table.pop();
        table
    }

    /// The added backslashes widen the cell. A bare visual's opaque
    /// bytes pass verbatim.
    fn cell(&self, cell: Cell) -> Cell {
        if cell.bare_visual {
            return cell;
        }
        Cell {
            width: cell.width + cell.rendered.matches('|').count(),
            rendered: render_md_tables::md_cell(&cell.rendered),
            bare_visual: false,
        }
    }
}
