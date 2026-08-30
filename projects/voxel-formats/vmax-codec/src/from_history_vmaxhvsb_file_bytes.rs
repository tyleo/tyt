use crate::{Error, Result, decompress_lzfse};
use vmax::VMaxHistoryVmaxhvsbFile;

/// Decodes `*.vmaxhvsb` history voxel-snapshot-buffer bytes (an LZFSE-framed
/// binary plist array of snapshots) into a [`VMaxHistoryVmaxhvsbFile`]. The
/// inverse of
/// [`to_history_vmaxhvsb_file_bytes`](crate::to_history_vmaxhvsb_file_bytes).
pub fn from_history_vmaxhvsb_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhvsbFile> {
    let decompressed = decompress_lzfse(bytes);
    plist::from_bytes(&decompressed).map_err(Error::Plist)
}
