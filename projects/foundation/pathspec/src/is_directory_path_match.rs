use crate::{GitIgnoreRegex, is_directory_match, path_match};

/// Matches a directory `path` down its ancestor directories, then the leaf as a
/// directory. An excluded ancestor prunes the subtree. `Some(true)` includes,
/// `Some(false)` excludes, `None` is no match.
pub fn is_directory_path_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    path_match(patterns, path, is_directory_match)
}
