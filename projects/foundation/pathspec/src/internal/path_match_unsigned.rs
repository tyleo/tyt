use crate::{UnsignedGitIgnoreRegex, ancestor_prefixes, is_directory_match_unsigned};

/// The shared ancestor walk of the unsigned path matchers: any ancestor match
/// includes, else `leaf_match` decides the leaf.
pub fn path_match_unsigned(
    patterns: &[UnsignedGitIgnoreRegex],
    path: &str,
    leaf_match: fn(&[UnsignedGitIgnoreRegex], &str) -> bool,
) -> bool {
    for prefix in ancestor_prefixes(path) {
        if is_directory_match_unsigned(patterns, prefix) {
            return true;
        }
    }

    leaf_match(patterns, path)
}

#[cfg(test)]
mod tests {
    use crate::{
        UnsignedGitIgnoreRegex, is_directory_path_match_unsigned, is_file_path_match_unsigned,
    };

    #[test]
    fn unsigned_path_match_is_any_match_along_the_path() {
        let patterns = UnsignedGitIgnoreRegex::from_spans(&["wall"]).unwrap();

        assert!(is_directory_path_match_unsigned(&patterns, "wall"));
        assert!(is_file_path_match_unsigned(&patterns, "wall/brick"));
        assert!(!is_file_path_match_unsigned(&patterns, "floor/tile"));
    }
}
