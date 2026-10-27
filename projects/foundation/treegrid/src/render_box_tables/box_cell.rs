/// `text` safe for one box table cell: newlines flattened to spaces.
pub fn box_cell(text: &str) -> String {
    text.replace(['\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use crate::render_box_tables;

    #[test]
    fn box_cell_flattens_newlines() {
        assert_eq!(render_box_tables::box_cell("a\nb\rc"), "a b c");
    }

    #[test]
    fn box_cell_keeps_pipes() {
        assert_eq!(render_box_tables::box_cell("a|b"), "a|b");
    }
}
