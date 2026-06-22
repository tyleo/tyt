use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Decodes `palette*.settings.vmaxpsb` bytes (a binary plist; unlike
/// `contents*.vmaxb` it is not LZFSE-framed) into a [`VMaxPaletteSettingsVmaxpsbFile`].
pub fn from_vmaxpsb_bytes(bytes: &[u8]) -> Result<VMaxPaletteSettingsVmaxpsbFile> {
    plist::from_bytes(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
