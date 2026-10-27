use crate::{GitIgnoreRegex, ancestor_prefixes, is_directory_match};

/// The shared ancestor walk of the signed path matchers: ancestors match as
/// directories with an excluded one pruning, then `leaf_match` decides the
/// leaf.
pub fn path_match(
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

#[cfg(test)]
mod tests {
    use crate::{GitIgnoreRegex, is_directory_path_match, is_file_path_match};

    #[test]
    fn selecting_a_node_selects_its_whole_subtree() {
        let patterns = GitIgnoreRegex::from_spans(&["wall"]).unwrap();

        assert_eq!(is_directory_path_match(&patterns, "wall"), Some(true));
        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(true));
        assert_eq!(is_directory_path_match(&patterns, "floor"), None);
        assert_eq!(is_file_path_match(&patterns, "floor/tile"), None);
    }

    #[test]
    fn a_selected_directory_pulls_in_its_object_subtree() {
        let patterns = GitIgnoreRegex::from_spans(&["*/"]).unwrap();

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
        let patterns = GitIgnoreRegex::from_spans(&["**", "!wall/", "wall/brick"]).unwrap();

        assert_eq!(is_directory_path_match(&patterns, "floor"), Some(true));
        assert_eq!(is_directory_path_match(&patterns, "wall"), Some(false));
        assert_eq!(is_file_path_match(&patterns, "wall/brick"), Some(false));
    }

    #[test]
    fn a_deeper_exclude_prunes_only_that_branch() {
        let patterns = GitIgnoreRegex::from_spans(&["house/", "!house/attic/"]).unwrap();

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
}
