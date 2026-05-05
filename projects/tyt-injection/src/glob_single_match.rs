use std::{
    fmt::Write,
    io::{Error, ErrorKind, Result},
    path::PathBuf,
};

/// Returns the single filesystem path matching `pattern`, erroring if zero or
/// more than one path matches.
pub fn glob_single_match(pattern: &str) -> Result<PathBuf> {
    let mut matches = Vec::new();
    for entry in glob::glob(pattern).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("invalid glob pattern '{pattern}': {e}"),
        )
    })? {
        matches.push(
            entry.map_err(|e| {
                Error::other(format!("error reading glob result: {e}"))
            })?,
        );
    }

    match matches.len() {
        0 => Err(Error::new(
            ErrorKind::NotFound,
            format!("missing file matching: {pattern}"),
        )),
        1 => Ok(matches.into_iter().next().unwrap()),
        n => {
            let mut msg = format!("multiple files ({n}) match '{pattern}':");
            for f in &matches {
                let _ = write!(msg, "\n  {}", f.display());
            }
            Err(Error::other(msg))
        }
    }
}
