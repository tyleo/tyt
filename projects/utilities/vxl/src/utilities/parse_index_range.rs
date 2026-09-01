use voxsmith::IndexRange;

/// Parses an index selector for a clap argument: one index such as `5`, or an
/// inclusive range `a-b` such as `2-5`, ordered start to end.
pub fn parse_index_range(text: &str) -> Result<IndexRange, String> {
    let (start, end) = match text.split_once('-') {
        Some((start, end)) => (parse_index(start)?, parse_index(end)?),
        None => {
            let index = parse_index(text)?;

            (index, index)
        }
    };

    IndexRange::new(start, end).map_err(|error| error.to_string())
}

/// Parses one non-negative index, naming the bad value on failure.
fn parse_index(text: &str) -> Result<usize, String> {
    text.parse::<usize>()
        .map_err(|_| format!("`{text}` is not an index"))
}

#[cfg(test)]
mod tests {
    use crate::parse_index_range;

    #[test]
    fn parses_single_index() {
        let range = parse_index_range("5").unwrap();

        assert!(range.contains(5));
        assert!(!range.contains(4));
        assert!(!range.contains(6));
    }

    #[test]
    fn parses_inclusive_range() {
        let range = parse_index_range("2-5").unwrap();

        assert!(!range.contains(1));
        assert!(range.contains(2));
        assert!(range.contains(5));
        assert!(!range.contains(6));
    }

    #[test]
    fn rejects_reversed_range() {
        assert!(parse_index_range("5-2").is_err());
    }

    #[test]
    fn rejects_non_integer_or_negative() {
        assert!(parse_index_range("x").is_err());
        assert!(parse_index_range("-3").is_err());
        assert!(parse_index_range("2-").is_err());
        assert!(parse_index_range("").is_err());
    }
}
