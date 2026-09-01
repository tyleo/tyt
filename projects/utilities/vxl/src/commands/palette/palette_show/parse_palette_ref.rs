use voxsmith::PaletteRef;

/// Parses the palette field of a `--property` selector: `*` for every
/// palette, else a non-negative index.
pub fn parse_palette_ref(text: &str) -> Result<PaletteRef, String> {
    if text == "*" {
        return Ok(PaletteRef::All);
    }

    text.parse::<usize>()
        .map(PaletteRef::Index)
        .map_err(|_| format!("`{text}` is not a palette index or `*`"))
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_palette_ref;
    use voxsmith::PaletteRef;

    #[test]
    fn parses_a_star_and_an_index() {
        assert_eq!(parse_palette_ref("*").unwrap(), PaletteRef::All);
        assert_eq!(parse_palette_ref("0").unwrap(), PaletteRef::Index(0));
        assert_eq!(parse_palette_ref("12").unwrap(), PaletteRef::Index(12));
    }

    #[test]
    fn rejects_a_non_index() {
        assert!(parse_palette_ref("a").is_err());
        assert!(parse_palette_ref("-1").is_err());
        assert!(parse_palette_ref("").is_err());
    }
}
