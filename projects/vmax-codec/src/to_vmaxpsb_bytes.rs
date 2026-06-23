use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxSerdePaletteSettingsVmaxpsbFile;

/// Encodes a [`VMaxSerdePaletteSettingsVmaxpsbFile`] into `palette*.settings.vmaxpsb` bytes
/// (a binary plist) — the inverse of
/// [`from_vmaxpsb_bytes`](crate::from_vmaxpsb_bytes).
pub fn to_vmaxpsb_bytes(file: &VMaxSerdePaletteSettingsVmaxpsbFile) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    plist::to_writer_binary(&mut out, file).map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    Ok(out)
}
