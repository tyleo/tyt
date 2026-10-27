use std::io::Result as IOResult;
use tempfile::TempDir;

/// Creates temporary directories.
pub trait CreateTempDir {
    /// A new empty directory, which the returned guard removes when it drops.
    fn create_temp_dir(&self) -> IOResult<TempDir>;
}
