use std::{fs, io::Result, path::Path};

/// Renames (moves) a file from `from` to `to`.
pub fn rename_file(from: &Path, to: &Path) -> Result<()> {
    fs::rename(from, to)
}
