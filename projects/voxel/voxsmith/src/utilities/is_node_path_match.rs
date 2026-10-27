use pathspec::{GitIgnoreRegex, is_directory_match, is_directory_path_match};

/// Whether `patterns` select the node at `path`. A matched ancestor does not
/// carry down as it does in [`is_directory_path_match`], so a match selects
/// the node alone. An excluded ancestor still blocks the node.
pub fn is_node_path_match(patterns: &[GitIgnoreRegex], path: &str) -> bool {
    if let Some((parent, _)) = path.rsplit_once('/')
        && is_directory_path_match(patterns, parent) == Some(false)
    {
        return false;
    }

    is_directory_match(patterns, path) == Some(true)
}
