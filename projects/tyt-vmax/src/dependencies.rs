use crate::Result;
use std::path::Path;

/// Dependencies for this crate's operations.
pub trait Dependencies {
    fn read_file(&self, path: &Path) -> Result<Vec<u8>>;
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<()>;
    fn write_stdout(&self, contents: &[u8]) -> Result<()>;
}
