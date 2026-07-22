use pathspec::{GitIgnoreRegex, is_directory_path_match, is_file_path_match};
use std::io::{Error, ErrorKind, Result};

/// Selects the candidates matched by the gitignore-style `patterns` with real
/// gitignore directory semantics: a matched directory pulls in its whole subtree
/// and an excluded directory prunes it. Each candidate is a `(path, is_dir)`
/// pair, and the returned flags line up with `candidates`. Use `match_paths` for
/// direct per-path matching without subtree recursion.
pub fn match_subtrees(patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>> {
    let compiled = GitIgnoreRegex::from_spans_ignore_inert(patterns)
        .map_err(|error| Error::new(ErrorKind::InvalidInput, error.to_string()))?;

    Ok(candidates
        .iter()
        .map(|(path, is_dir)| {
            let matched = if *is_dir {
                is_directory_path_match(&compiled, path)
            } else {
                is_file_path_match(&compiled, path)
            };
            matched == Some(true)
        })
        .collect())
}
