use std::path::PathBuf;

/// The locations preferences load from.
///
/// [`resolve_prefs_paths`](crate::resolve_prefs_paths) builds one from the
/// current working directory.
#[derive(Clone, Debug)]
pub struct PrefsPaths {
    /// Directory the hierarchy walk ends at.
    pub cwd: PathBuf,

    /// Root of the git repository containing `cwd`, or `None` outside a
    /// repository.
    pub git_root: Option<PathBuf>,

    /// User home directory, or `None` if it cannot be determined.
    pub user: Option<PathBuf>,
}
