/// The palette field of a `--property` selector: one palette by index, or
/// every palette.
#[derive(Clone, Debug, PartialEq)]
pub enum PaletteRef {
    /// Every palette in the document.
    All,
    /// One palette by its index into the document's palettes.
    Index(usize),
}

impl PaletteRef {
    /// Parses the palette field: `*` for every palette, else a non-negative
    /// index.
    pub fn parse(value: &str) -> Result<Self, String> {
        if value == "*" {
            return Ok(PaletteRef::All);
        }
        value
            .parse::<usize>()
            .map(PaletteRef::Index)
            .map_err(|_| format!("`{value}` is not a palette index or `*`"))
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::PaletteRef;

    #[test]
    fn parses_a_star_and_an_index() {
        assert_eq!(PaletteRef::parse("*").unwrap(), PaletteRef::All);
        assert_eq!(PaletteRef::parse("0").unwrap(), PaletteRef::Index(0));
        assert_eq!(PaletteRef::parse("12").unwrap(), PaletteRef::Index(12));
    }

    #[test]
    fn rejects_a_non_index() {
        assert!(PaletteRef::parse("a").is_err());
        assert!(PaletteRef::parse("-1").is_err());
        assert!(PaletteRef::parse("").is_err());
    }
}
