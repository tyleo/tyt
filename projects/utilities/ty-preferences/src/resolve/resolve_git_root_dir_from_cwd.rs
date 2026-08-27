use std::{
    io::{Error as IOError, ErrorKind, Result as IOResult},
    path::{Path, PathBuf},
    process::Command,
};

/// Resolves the root directory of the git repository containing `cwd`, or
/// `None` when `cwd` is not in a repository.
pub fn resolve_git_root_dir_from_cwd(cwd: &Path) -> IOResult<Option<PathBuf>> {
    let output = match Command::new("git")
        .arg("-C")
        .arg(cwd)
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
        String::from_utf8(output.stdout).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;

    Ok(Some(PathBuf::from(path.trim())))
}
