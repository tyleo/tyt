use crate::UnsignedGitIgnoreRegex;

/// Whether any pattern matches `path` as a directory.
pub fn is_directory_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(path))
}
