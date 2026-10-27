use crate::render::Cell;

/// How a table layout draws its headings, tables, and cells.
pub trait TableFrame {
    /// The heading of a section `depth` levels down.
    fn heading(&self, depth: usize, text: &str) -> String;

    /// One table over `headers` and `rows`, with no trailing newline.
    fn table(&self, headers: &[Cell], rows: &[Vec<Cell>]) -> String;

    /// `cell` made safe for one table cell.
    fn cell(&self, cell: Cell) -> Cell;
}
