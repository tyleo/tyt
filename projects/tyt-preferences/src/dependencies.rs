use std::{
    io,
    path::{Path, PathBuf},
};

/// Dependency injection for preference I/O operations.
pub trait Dependencies {
    /// Returns the user home directory, or `None` if it cannot be determined.
    fn user_home_dir(&self) -> io::Result<Option<PathBuf>>;

    /// Returns the root directory of the current git repository, or `None` if not in a repository.
    fn git_root_dir(&self) -> io::Result<Option<PathBuf>>;

    /// Reads the contents of a file, or returns `None` if the file does not exist.
    fn read_file(&self, path: &Path) -> io::Result<Option<Vec<u8>>>;
}
