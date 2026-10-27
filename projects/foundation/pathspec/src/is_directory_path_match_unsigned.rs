use crate::{UnsignedGitIgnoreRegex, is_directory_match_unsigned, path_match_unsigned};

/// Whether any pattern matches an ancestor of the directory `path` or the leaf
/// as a directory.
pub fn is_directory_path_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    path_match_unsigned(patterns, path, is_directory_match_unsigned)
}
