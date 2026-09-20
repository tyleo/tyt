use std::io::Result as IOResult;
use ty_preferences::PrefsPaths;

/// Resolves the directories `.vxlconfig` loads from.
pub trait ResolvePrefsPaths {
    /// The working directory, the git root holding it, and the user's home.
    fn resolve_prefs_paths(&self) -> IOResult<PrefsPaths>;
}
