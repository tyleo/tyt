use crate::{DecodeVMaxPlist, DecompressLzfse, Error, Result, decompress_lzfse_or_raw};
use vmax::VMaxHistoryVmaxhvsbFile;

/// Decodes `*.vmaxhvsb` history voxel-snapshot-buffer bytes (an LZFSE-framed
/// binary plist array of snapshots) into a [`VMaxHistoryVmaxhvsbFile`]
/// through `dependencies`. The inverse of
/// [`to_history_vmaxhvsb_file_bytes`](crate::to_history_vmaxhvsb_file_bytes).
pub fn from_history_vmaxhvsb_file_bytes<D: DecompressLzfse + DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxHistoryVmaxhvsbFile> {
    let plist_bytes = decompress_lzfse_or_raw(dependencies, bytes);
    dependencies
        .decode_history_vmaxhvsb(&plist_bytes)
        .map_err(Error::Plist)
}
