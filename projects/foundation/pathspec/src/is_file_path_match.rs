use crate::{GitIgnoreRegex, is_file_match, path_match};

/// Matches a file `path` down its ancestor directories, then the leaf as a
/// file, so a directory-only pattern never matches the leaf by name. An
/// excluded ancestor prunes the subtree. `Some(true)` includes, `Some(false)`
/// excludes, `None` is no match.
pub fn is_file_path_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    path_match(patterns, path, is_file_match)
}

#[cfg(test)]
mod tests {
    use crate::{GitIgnoreRegex, is_directory_path_match, is_file_path_match};

    #[test]
    fn a_directory_only_pattern_does_not_match_a_bare_object() {
        // A trailing slash matches a node by name but never a leaf object by the
        // same name.
        let patterns = GitIgnoreRegex::from_spans(&["brick/"]).unwrap();

        assert_eq!(is_directory_path_match(&patterns, "brick"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "brick"), None);
    }

    #[test]
    fn a_file_leaf_can_be_selected_by_name() {
        let patterns = GitIgnoreRegex::from_spans(&["**/brick"]).unwrap();

        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "wall/stone"), None);
    }
}
