use crate::{Error, Result};
use vmax::VMaxSerdePaletteSettingsVmaxpsbFile;

/// Encodes a [`VMaxSerdePaletteSettingsVmaxpsbFile`] into
/// `palette*.settings.vmaxpsb` bytes (a binary plist), the inverse of
/// [`from_palette_settings_vmaxpsb_file_bytes`](crate::from_palette_settings_vmaxpsb_file_bytes).
pub fn to_palette_settings_vmaxpsb_file_bytes(
    file: &VMaxSerdePaletteSettingsVmaxpsbFile,
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    plist::to_writer_binary(&mut out, file).map_err(Error::Plist)?;
    Ok(out)
}
