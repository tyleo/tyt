use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxSerdePaletteSettingsVmaxpsbFile;

/// Decodes `palette*.settings.vmaxpsb` bytes (a binary plist) into a
/// [`VMaxSerdePaletteSettingsVmaxpsbFile`].
pub fn from_palette_settings_vmaxpsb_file_bytes(
    bytes: &[u8],
) -> Result<VMaxSerdePaletteSettingsVmaxpsbFile> {
    plist::from_bytes(bytes).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
