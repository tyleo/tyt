use crate::{resolve_cwd, resolve_git_root_dir_from_cwd};
use std::{io::Result as IOResult, path::PathBuf};

/// Resolves the root directory of the git repository containing the current
/// working directory, or `None` when outside a repository.
pub fn resolve_git_root_dir() -> IOResult<Option<PathBuf>> {
    resolve_git_root_dir_from_cwd(&resolve_cwd()?)
}
