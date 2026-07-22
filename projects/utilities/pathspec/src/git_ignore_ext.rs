use crate::{GitIgnoreRegex, GitIgnoreRegexKind, UnsignedGitIgnoreRegex};

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

/// Matches a directory `path` down its ancestor directories, then the leaf as a
/// directory. An excluded ancestor prunes the subtree. `Some(true)` includes,
/// `Some(false)` excludes, `None` is no match.
pub fn is_directory_path_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    path_match(patterns, path, is_directory_match)
}

/// Matches a file `path` down its ancestor directories, then the leaf as a
/// file, so a directory-only pattern never matches the leaf by name. An
/// excluded ancestor prunes the subtree. `Some(true)` includes, `Some(false)`
/// excludes, `None` is no match.
pub fn is_file_path_match(patterns: &[GitIgnoreRegex], path: &str) -> Option<bool> {
    path_match(patterns, path, is_file_match)
}

/// The shared ancestor walk of the signed path matchers: ancestors match as
/// directories with an excluded one pruning, then `leaf_match` decides the
/// leaf.
fn path_match(
    patterns: &[GitIgnoreRegex],
    path: &str,
    leaf_match: fn(&[GitIgnoreRegex], &str) -> Option<bool>,
) -> Option<bool> {
    let mut directory = None;

    for prefix in ancestor_prefixes(path) {
        if let Some(sign) = is_directory_match(patterns, prefix) {
            directory = Some(sign);
        }

        if directory == Some(false) {
            return Some(false);
        }
    }

    let leaf = leaf_match(patterns, path);

    if leaf == Some(false) {
        return Some(false);
    }

    match (directory, leaf) {
        (None, None) => None,
        _ => Some(true),
    }
}

/// Whether any pattern matches `path` as a directory.
pub fn is_directory_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    patterns.iter().any(|pattern| pattern.is_match(path))
}

/// Whether any file pattern matches `path`, skipping directory-only patterns.
pub fn is_file_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    patterns
        .iter()
        .any(|pattern| pattern.kind() != GitIgnoreRegexKind::Directory && pattern.is_match(path))
}

/// Whether any pattern matches an ancestor of the directory `path` or the leaf
/// as a directory.
pub fn is_directory_path_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    path_match_unsigned(patterns, path, is_directory_match_unsigned)
}

/// Whether any pattern matches an ancestor of the file `path`, or any file
/// pattern matches the leaf.
pub fn is_file_path_match_unsigned(patterns: &[UnsignedGitIgnoreRegex], path: &str) -> bool {
    path_match_unsigned(patterns, path, is_file_match_unsigned)
}

/// The shared ancestor walk of the unsigned path matchers: any ancestor match
/// includes, else `leaf_match` decides the leaf.
fn path_match_unsigned(
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

/// The proper ancestor prefixes of `path`, shallowest first. `a/b/c` gives `a`
/// then `a/b`.
fn ancestor_prefixes(path: &str) -> impl Iterator<Item = &str> {
    path.match_indices('/')
        .map(move |(index, _)| &path[..index])
}

#[cfg(test)]
mod tests {
    use crate::{
        GitIgnoreRegex, UnsignedGitIgnoreRegex, is_directory_match, is_directory_path_match,
        is_directory_path_match_unsigned, is_file_match, is_file_path_match,
        is_file_path_match_unsigned,
    };

    fn signed(patterns: &[&str]) -> Vec<GitIgnoreRegex> {
        GitIgnoreRegex::from_spans_ignore_inert(patterns).unwrap()
    }

    fn unsigned(patterns: &[&str]) -> Vec<UnsignedGitIgnoreRegex> {
        UnsignedGitIgnoreRegex::from_spans_ignore_inert(patterns).unwrap()
    }

    #[test]
    fn the_last_matching_pattern_wins() {
        let patterns = signed(&["*", "!secret"]);

        assert_eq!(is_directory_match(&patterns, "public"), Some(true));
        assert_eq!(is_directory_match(&patterns, "secret"), Some(false));

        // Reversing the order flips the overlapping path.
        let reversed = signed(&["!secret", "*"]);

        assert_eq!(is_directory_match(&reversed, "secret"), Some(true));
    }

    #[test]
    fn file_matching_skips_directory_only_patterns() {
        let patterns = signed(&["logs/"]);

        assert_eq!(is_file_match(&patterns, "logs"), None);
        assert_eq!(is_directory_match(&patterns, "logs"), Some(true));
    }

    #[test]
    fn selecting_a_node_selects_its_whole_subtree() {
        let patterns = signed(&["wall"]);

        assert_eq!(is_directory_path_match(&patterns, "wall"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(true));
        assert_eq!(is_directory_path_match(&patterns, "floor"), None);
        assert_eq!(is_file_path_match(&patterns, "floor/tile"), None);
    }

    #[test]
    fn a_directory_only_pattern_does_not_match_a_bare_object() {
        // A trailing slash matches a node by name but never a leaf object by the
        // same name.
        let patterns = signed(&["brick/"]);

        assert_eq!(is_directory_path_match(&patterns, "brick"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "brick"), None);
    }

    #[test]
    fn a_selected_directory_pulls_in_its_object_subtree() {
        let patterns = signed(&["*/"]);

        assert_eq!(is_directory_path_match(&patterns, "wall"), Some(true));
        // An object rides in as part of a selected node's subtree.
        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(true));
        // A top-level object has no selected ancestor, so it stays out.
        assert_eq!(is_file_path_match(&patterns, "loose"), None);
    }

    #[test]
    fn an_excluded_directory_blocks_reincluding_its_contents() {
        // Once `wall/` is excluded, nothing under it returns, even a later
        // pattern naming a descendant.
        let patterns = signed(&["**", "!wall/", "wall/brick"]);

        assert_eq!(is_directory_path_match(&patterns, "floor"), Some(true));
        assert_eq!(is_directory_path_match(&patterns, "wall"), Some(false));
        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(false));
    }

    #[test]
    fn a_deeper_exclude_prunes_only_that_branch() {
        let patterns = signed(&["house/", "!house/attic/"]);

        assert_eq!(is_directory_path_match(&patterns, "house"), Some(true));
        assert_eq!(is_directory_path_match(&patterns, "house/room"), Some(true));
        assert_eq!(
            is_directory_path_match(&patterns, "house/attic"),
            Some(false)
        );
        assert_eq!(
            is_file_path_match(&patterns, "house/attic/box"),
            Some(false)
        );
    }

    #[test]
    fn a_file_leaf_can_be_selected_by_name() {
        let patterns = signed(&["**/brick"]);

        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "wall/stone"), None);
    }

    #[test]
    fn unsigned_path_match_is_any_match_along_the_path() {
        let patterns = unsigned(&["wall"]);

        assert!(is_directory_path_match_unsigned(&patterns, "wall"));
        assert!(is_file_path_match_unsigned(&patterns, "wall/brick"));
        assert!(!is_file_path_match_unsigned(&patterns, "floor/tile"));
    }

    #[test]
    fn an_unsigned_directory_only_pattern_does_not_match_a_file_leaf() {
        let patterns = unsigned(&["logs/"]);

        assert!(is_directory_path_match_unsigned(&patterns, "logs"));
        assert!(!is_file_path_match_unsigned(&patterns, "logs"));
        // A file under the directory still rides in through the ancestor.
        assert!(is_file_path_match_unsigned(&patterns, "logs/app"));
    }
}
