use crate::{Error, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Decodes `palette*.settings.vmaxpsb` bytes (a binary plist) into a
/// [`VMaxPaletteSettingsVmaxpsbFile`].
pub fn from_palette_settings_vmaxpsb_file_bytes(
    bytes: &[u8],
) -> Result<VMaxPaletteSettingsVmaxpsbFile> {
    plist::from_bytes(bytes).map_err(Error::Plist)
}
