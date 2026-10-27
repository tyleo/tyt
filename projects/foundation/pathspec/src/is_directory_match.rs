use crate::GitIgnoreRegex;

/// Matches `path` as a directory across the patterns, last match winning.
/// `Some(true)` includes, `Some(false)` excludes, `None` is no match.
pub fn is_directory_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    let mut result = None;

    for pattern in patterns {
        if let Some(sign) = pattern.is_match(path) {
            result = Some(sign);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::{GitIgnoreRegex, is_directory_match};

    #[test]
    fn the_last_matching_pattern_wins() {
        let patterns = GitIgnoreRegex::from_spans(&["*", "!secret"]).unwrap();

        assert_eq!(is_directory_match(&patterns, "public"), Some(true));
        assert_eq!(is_directory_match(&patterns, "secret"), Some(false));

        // Reversing the order flips the overlapping path.
        let reversed = GitIgnoreRegex::from_spans(&["!secret", "*"]).unwrap();

        assert_eq!(is_directory_match(&reversed, "secret"), Some(true));
    }
}
