use branded_id::U32Id;
use voxsmith::utilities::IdSelector;

/// Parses an id selector for a clap argument: `*` for every entry, one id
/// such as `5`, or an inclusive range `a-b` such as `2-5`.
pub fn parse_id_selector<TBrand>(text: &str) -> Result<IdSelector<TBrand>, String> {
    if text == "*" {
        return Ok(IdSelector::all());
    }

    let Some((start, end)) = text.split_once('-') else {
        return Ok(IdSelector::id(parse_id(text)?));
    };

    IdSelector::range(parse_id(start)?..=parse_id(end)?).map_err(|error| error.to_string())
}

/// Parses one id, quoting the bad value on failure.
fn parse_id<TBrand>(text: &str) -> Result<U32Id<TBrand>, String> {
    text.parse().map_err(|_| format!("`{text}` is not an id"))
}

#[cfg(test)]
mod tests {
    use crate::parse_id_selector;
    use branded_id::U32Id;
    use voxsmith::utilities::IdSelector;

    struct BEntry;

    fn parse(text: &str) -> Result<IdSelector<BEntry>, String> {
        parse_id_selector(text)
    }

    fn id(value: u32) -> U32Id<BEntry> {
        U32Id::from_u32(value)
    }

    #[test]
    fn parses_every_entry() {
        assert_eq!(parse("*"), Ok(IdSelector::all()));
    }

    #[test]
    fn parses_a_single_id() {
        assert_eq!(parse("5"), Ok(IdSelector::id(id(5))));
    }

    #[test]
    fn parses_an_inclusive_range() {
        assert_eq!(parse("2-5"), Ok(IdSelector::range(id(2)..=id(5)).unwrap()));
    }

    #[test]
    fn parses_the_largest_id() {
        assert_eq!(parse("4294967295"), Ok(IdSelector::id(U32Id::MAX)));
    }

    #[test]
    fn rejects_a_reversed_range() {
        assert_eq!(
            parse("5-2"),
            Err("range start 5 is greater than its end 2".to_owned())
        );
    }

    #[test]
    fn rejects_an_id_past_the_id_width() {
        assert_eq!(
            parse("4294967296"),
            Err("`4294967296` is not an id".to_owned())
        );
    }

    #[test]
    fn rejects_anything_else() {
        for text in ["x", "-3", "2-", "", "**", "*-2", "1-*"] {
            assert!(parse(text).is_err(), "{text}");
        }
    }
}
