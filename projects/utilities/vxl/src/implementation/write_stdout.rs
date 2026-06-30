use crate::Result;
use std::io::{self, Write};

/// Writes `contents` to standard output.
pub fn write_stdout(contents: &[u8]) -> Result<()> {
    io::stdout().write_all(contents)?;
    Ok(())
}
