use crate::{PrefsPaths, resolve_git_root_dir_from_cwd, resolve_user_home_dir};
use std::{io::Result as IOResult, path::PathBuf};

/// Resolves the locations preferences load from: `cwd`, the root of the git
/// repository containing it, and the user home directory.
pub fn resolve_prefs_paths_from_cwd(cwd: PathBuf) -> IOResult<PrefsPaths> {
    let git_root = resolve_git_root_dir_from_cwd(&cwd)?;

    let user = resolve_user_home_dir();

    Ok(PrefsPaths {
        cwd,
        git_root,
        user,
    })
}
