use globset::GlobBuilder;
use std::io::{Error, ErrorKind, Result};

pub fn match_glob(pattern: &str, candidates: &[&str]) -> Result<Vec<bool>> {
    let glob = GlobBuilder::new(pattern)
        .literal_separator(true)
        .build()
        .map_err(|e| Error::new(ErrorKind::InvalidInput, e))?
        .compile_matcher();
    Ok(candidates.iter().map(|c| glob.is_match(c)).collect())
}
