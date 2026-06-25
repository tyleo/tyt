use std::{fs, io::Result, path::Path};

pub fn write_file(path: &Path, contents: &[u8]) -> Result<()> {
    fs::write(path, contents)
}
