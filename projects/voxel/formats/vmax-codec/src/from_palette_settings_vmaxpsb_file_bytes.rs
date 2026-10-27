use crate::{DecodeVMaxPlist, Error, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Decodes `palette*.settings.vmaxpsb` bytes (a binary plist) into a
/// [`VMaxPaletteSettingsVmaxpsbFile`] through `dependencies`.
pub fn from_palette_settings_vmaxpsb_file_bytes<D: DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxPaletteSettingsVmaxpsbFile> {
    dependencies
        .decode_palette_settings_vmaxpsb(bytes)
        .map_err(Error::Plist)
}
