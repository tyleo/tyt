use std::io::{Result, Write};

pub fn write_stdout(contents: &[u8]) -> Result<()> {
    std::io::stdout().write_all(contents)
}
