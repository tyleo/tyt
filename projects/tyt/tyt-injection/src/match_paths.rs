use pathspec::{GitIgnoreRegex, is_directory_match, is_file_match};
use std::io::{Error, ErrorKind, Result};

/// Selects the candidates matched by the gitignore-style `patterns`. Each
/// candidate is a `(path, is_dir)` pair matched directly against the patterns,
/// last match winning; a directory-only pattern skips file candidates. A whole
/// subtree is selected explicitly with a `dir/**` pattern. The returned flags
/// line up with `candidates`.
pub fn match_paths(patterns: &[&str], candidates: &[(&str, bool)]) -> Result<Vec<bool>> {
    let compiled = GitIgnoreRegex::from_spans_ignore_inert(patterns)
        .map_err(|error| Error::new(ErrorKind::InvalidInput, error.to_string()))?;

    Ok(candidates
        .iter()
        .map(|(path, is_dir)| {
            let matched = if *is_dir {
                is_directory_match(&compiled, path)
            } else {
                is_file_match(&compiled, path)
            };
            matched == Some(true)
        })
        .collect())
}
