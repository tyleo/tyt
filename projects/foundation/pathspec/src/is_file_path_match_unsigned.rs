use crate::{UnsignedGitIgnoreRegex, is_file_match_unsigned, path_match_unsigned};

/// Whether any pattern matches an ancestor of the file `path`, or any file
/// pattern matches the leaf.
pub fn is_file_path_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    path_match_unsigned(patterns, path, is_file_match_unsigned)
}

#[cfg(test)]
mod tests {
    use crate::{
        UnsignedGitIgnoreRegex, is_directory_path_match_unsigned, is_file_path_match_unsigned,
    };

    #[test]
    fn an_unsigned_directory_only_pattern_does_not_match_a_file_leaf() {
        let patterns = UnsignedGitIgnoreRegex::from_spans(&["logs/"]).unwrap();

        assert!(is_directory_path_match_unsigned(&patterns, "logs"));
        assert!(!is_file_path_match_unsigned(&patterns, "logs"));
        // A file under the directory still rides in through the ancestor.
        assert!(is_file_path_match_unsigned(&patterns, "logs/app"));
    }
}
