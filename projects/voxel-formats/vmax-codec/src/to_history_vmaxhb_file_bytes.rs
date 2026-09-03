use crate::{CompressLzfse, EncodeVMaxPlist, Error, Result};
use vmax::VMaxHistoryVmaxhbFile;

/// Encodes a [`VMaxHistoryVmaxhbFile`] into `*.vmaxhb` bytes (a binary plist
/// wrapped in an LZFSE block stream) through `dependencies`, the inverse of
/// [`from_history_vmaxhb_file_bytes`](crate::from_history_vmaxhb_file_bytes).
pub fn to_history_vmaxhb_file_bytes<D: CompressLzfse + EncodeVMaxPlist>(
    dependencies: &D,
    file: &VMaxHistoryVmaxhbFile,
) -> Result<Vec<u8>> {
    let plist_bytes = dependencies
        .encode_history_vmaxhb(file)
        .map_err(Error::Plist)?;
    Ok(dependencies.compress_lzfse(&plist_bytes))
}
