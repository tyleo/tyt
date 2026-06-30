use crate::implementation;

/// One aligned Markdown table: a header row, a dash separator, then the rows,
/// every column padded to its widest cell and at least three wide so the
/// separator stays valid Markdown. A row shorter than the headers leaves its
/// trailing columns blank. Width is measured past ANSI escapes, so colored
/// swatch cells line up too.
pub(crate) fn markdown_table(headers: &[&str], rows: &[Vec<String>]) -> String {
    let mut widths: Vec<usize> = headers
        .iter()
        .map(|header| implementation::visible_width(header).max(3))
        .collect();
    for row in rows {
        for (column, cell) in row.iter().enumerate().take(widths.len()) {
            widths[column] = widths[column].max(implementation::visible_width(cell));
        }
    }

    let header_cells: Vec<String> = headers.iter().map(|header| header.to_string()).collect();
    let separators: Vec<String> = widths.iter().map(|width| "-".repeat(*width)).collect();
    let mut output = table_row(&header_cells, &widths);
    output.push_str(&table_row(&separators, &widths));
    for row in rows {
        output.push_str(&table_row(row, &widths));
    }
    output
}

/// One `| ... | ... |` row, each column padded to its width, a missing trailing
/// column left blank.
fn table_row(cells: &[String], widths: &[usize]) -> String {
    let mut line = String::from("|");
    for (column, width) in widths.iter().enumerate() {
        let cell = cells.get(column).map(String::as_str).unwrap_or("");
        line.push(' ');
        line.push_str(&implementation::pad_right(cell, *width));
        line.push_str(" |");
    }
    line.push('\n');
    line
}
