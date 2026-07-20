/// `text` safe for one table cell: pipes escaped, newlines flattened to spaces.
pub(crate) fn markdown_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use crate::render_tables;

    #[test]
    fn markdown_cell_escapes_pipes() {
        assert_eq!(render_tables::markdown_cell("a|b"), "a\\|b");
    }

    #[test]
    fn markdown_cell_flattens_newlines() {
        assert_eq!(render_tables::markdown_cell("a\nb\rc"), "a b c");
    }
}
