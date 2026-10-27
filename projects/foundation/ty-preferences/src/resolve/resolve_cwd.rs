use std::{env, io::Result as IOResult, path::PathBuf};

/// Resolves the current working directory.
pub fn resolve_cwd() -> IOResult<PathBuf> {
    env::current_dir()
}
