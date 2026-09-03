use crate::{DecodeVMaxPlist, DecompressLzfse, Error, Result, decompress_lzfse_or_raw};
use vmax::VMaxHistoryVmaxhbFile;

/// Decodes `*.vmaxhb` undo-history bytes (an LZFSE-framed binary plist) into a
/// [`VMaxHistoryVmaxhbFile`] through `dependencies`. The inverse of
/// [`to_history_vmaxhb_file_bytes`](crate::to_history_vmaxhb_file_bytes).
pub fn from_history_vmaxhb_file_bytes<D: DecompressLzfse + DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxHistoryVmaxhbFile> {
    let plist_bytes = decompress_lzfse_or_raw(dependencies, bytes);
    dependencies
        .decode_history_vmaxhb(&plist_bytes)
        .map_err(Error::Plist)
}
