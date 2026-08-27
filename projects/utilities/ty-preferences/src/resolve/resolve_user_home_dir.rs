use std::{env, path::PathBuf};

/// Resolves the user home directory, or `None` if it cannot be determined.
pub fn resolve_user_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
