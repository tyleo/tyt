/// `text` safe for one table cell: pipes escaped, newlines flattened to spaces.
pub fn md_cell(text: &str) -> String {
    text.replace('|', "\\|").replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use crate::render_md_tables;

    #[test]
    fn md_cell_escapes_pipes() {
        assert_eq!(render_md_tables::md_cell("a|b"), "a\\|b");
    }

    #[test]
    fn md_cell_flattens_newlines() {
        assert_eq!(render_md_tables::md_cell("a\nb\rc"), "a b c");
    }
}
