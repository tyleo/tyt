use crate::list_dir;
use pathspec::UnsignedGitIgnoreRegex;
use std::{
    fmt::Write,
    io::{Error, ErrorKind, Result},
    path::{Path, PathBuf},
};

/// Returns the single filesystem path matching `pattern`, erroring if zero or
/// more than one path matches. The final segment is a gitignore-style glob
/// matched against the names in its parent directory; any leading directory is
/// taken literally.
pub fn glob_single_match(pattern: &str) -> Result<PathBuf> {
    let pattern_path = Path::new(pattern);

    let Some(name) = pattern_path.file_name().and_then(|name| name.to_str()) else {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            format!("invalid glob pattern '{pattern}'"),
        ));
    };

    let matcher = UnsignedGitIgnoreRegex::from_span(name).map_err(|error| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("invalid glob pattern '{pattern}': {error}"),
        )
    })?;

    // A pattern with no directory part lists the current directory.
    let dir = pattern_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    let listing_dir = dir.unwrap_or(Path::new("."));

    // A missing directory means nothing matches, the same as an empty listing.
    let entries = match list_dir(listing_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };

    let mut matches = Vec::new();
    for entry in entries {
        let Some(file_name) = entry.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if matcher.is_match(file_name) {
            matches.push(match dir {
                Some(dir) => dir.join(file_name),
                None => PathBuf::from(file_name),
            });
        }
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
