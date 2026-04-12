use crate::{Dependencies, Result};
use std::io::{Error as IOError, ErrorKind};

/// Runs `magick identify -format {format} {path}` and parses the output as `u32`.
pub fn identify_u32(deps: &impl Dependencies, path: &str, format: &str) -> Result<u32> {
    let output = deps.exec_magick(["identify", "-format", format, path])?;
    let text = String::from_utf8_lossy(&output);
    text.trim().parse::<u32>().map_err(|_| {
        IOError::new(
            ErrorKind::InvalidData,
            format!("invalid u32 from identify {format} on {path}: {text}"),
        )
        .into()
    })
}
