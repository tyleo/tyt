use crate::{Error, Result};

/// Parses an index token of `flag`, a `u32` counted from `0`.
pub(crate) fn parse_flag_index(flag: &str, text: &str) -> Result<u32> {
    text.parse::<u32>().map_err(|_| {
        Error::usage(format!(
            "{flag} takes an index counted from 0, not `{text}`"
        ))
    })
}

#[cfg(test)]
mod tests {
    use crate::commands::parse_flag_index;

    #[test]
    fn parses_a_count_from_zero_and_rejects_the_rest() {
        assert_eq!(parse_flag_index("--flag", "0").unwrap(), 0);
        assert_eq!(parse_flag_index("--flag", "12").unwrap(), 12);
        assert!(parse_flag_index("--flag", "-1").is_err());
        assert!(parse_flag_index("--flag", "one").is_err());
    }
}
