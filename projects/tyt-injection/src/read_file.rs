use std::{io::Result, path::Path};

pub fn read_file(path: &Path) -> Result<Vec<u8>> {
    std::fs::read(path)
}
