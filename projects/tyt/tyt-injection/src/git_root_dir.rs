use std::{
    io::{Error, ErrorKind, Result},
    path::PathBuf,
    process::Command,
};

/// Returns the root directory of the current git repository, or `None` when
/// outside a repository or git is not installed.
pub fn git_root_dir() -> Result<Option<PathBuf>> {
    let output = match Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        Ok(output) => output,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };

    if !output.status.success() {
        return Ok(None);
    }

    let path =
        String::from_utf8(output.stdout).map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    Ok(Some(PathBuf::from(path.trim())))
}
