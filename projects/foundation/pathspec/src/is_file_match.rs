use crate::{GitIgnoreRegex, GitIgnoreRegexKind};

/// Matches `path` as a file, skipping directory-only patterns, last match
/// winning. `Some(true)` includes, `Some(false)` excludes, `None` is no match.
pub fn is_file_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    let mut result = None;

    for pattern in patterns {
        if pattern.unsigned().kind() == GitIgnoreRegexKind::Directory {
            continue;
        }

        if let Some(sign) = pattern.is_match(path) {
            result = Some(sign);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use crate::{GitIgnoreRegex, is_directory_match, is_file_match};

    #[test]
    fn file_matching_skips_directory_only_patterns() {
        let patterns = GitIgnoreRegex::from_spans(&["logs/"]).unwrap();

        assert_eq!(is_file_match(&patterns, "logs"), None);
        assert_eq!(is_directory_match(&patterns, "logs"), Some(true));
    }
}
