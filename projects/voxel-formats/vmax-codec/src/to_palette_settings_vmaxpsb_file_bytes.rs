use crate::{EncodeVMaxPlist, Error, Result};
use vmax::VMaxPaletteSettingsVmaxpsbFile;

/// Encodes a [`VMaxPaletteSettingsVmaxpsbFile`] into
/// `palette*.settings.vmaxpsb` bytes (a binary plist) through `dependencies`,
/// the inverse of
/// [`from_palette_settings_vmaxpsb_file_bytes`](crate::from_palette_settings_vmaxpsb_file_bytes).
pub fn to_palette_settings_vmaxpsb_file_bytes<D: EncodeVMaxPlist>(
    dependencies: &D,
    file: &VMaxPaletteSettingsVmaxpsbFile,
) -> Result<Vec<u8>> {
    dependencies
        .encode_palette_settings_vmaxpsb(file)
        .map_err(Error::Plist)
}
