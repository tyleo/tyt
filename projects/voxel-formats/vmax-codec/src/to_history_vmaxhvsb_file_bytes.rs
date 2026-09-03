use crate::{CompressLzfse, EncodeVMaxPlist, Error, Result};
use vmax::VMaxHistoryVmaxhvsbFile;

/// Encodes a [`VMaxHistoryVmaxhvsbFile`] into `*.vmaxhvsb` bytes (a binary
/// plist wrapped in an LZFSE block stream) through `dependencies`, the inverse
/// of
/// [`from_history_vmaxhvsb_file_bytes`](crate::from_history_vmaxhvsb_file_bytes).
pub fn to_history_vmaxhvsb_file_bytes<D: CompressLzfse + EncodeVMaxPlist>(
    dependencies: &D,
    file: &VMaxHistoryVmaxhvsbFile,
) -> Result<Vec<u8>> {
    let plist_bytes = dependencies
        .encode_history_vmaxhvsb(file)
        .map_err(Error::Plist)?;
    Ok(dependencies.compress_lzfse(&plist_bytes))
}
