use std::{io::Result, path::Path};

pub fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    std::fs::write(path, contents)
}
