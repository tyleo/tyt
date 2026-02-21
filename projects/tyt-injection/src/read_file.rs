use std::{fs, io::Result, path::Path};

pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    fs::read(path)
}
