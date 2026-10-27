use crate::{GitIgnoreRegexKind, UnsignedGitIgnoreRegex};

/// Whether any file pattern matches `path`, skipping directory-only patterns.
pub fn is_file_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    patterns
        .iter()
        .any(|pattern| pattern.kind() != GitIgnoreRegexKind::Directory && pattern.is_match(path))
}
