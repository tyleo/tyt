use crate::{PrefsPaths, resolve_cwd, resolve_prefs_paths_from_cwd};
use std::io::Result as IOResult;

/// Resolves the locations preferences load from. The current working directory
/// supplies `cwd`.
pub fn resolve_prefs_paths() -> IOResult<PrefsPaths> {
    resolve_prefs_paths_from_cwd(resolve_cwd()?)
}
