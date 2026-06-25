use crate::Result;
use std::path::{Path, PathBuf};

/// Dependencies for meta operations.
pub trait Dependencies {
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()>;
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> Result<String>;
    fn write<P: AsRef<Path>>(&self, path: P, contents: &str) -> Result<()>;
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
    fn workspace_root(&self) -> Result<PathBuf>;
}
