use std::{env, path::PathBuf};

/// Returns the user home directory from `HOME` or `USERPROFILE`, or `None`
/// if neither is set.
pub fn user_home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}
