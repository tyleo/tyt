use crate::{Error, Result, decompress_lzfse};
use vmax::VMaxHistoryVmaxhbFile;

/// Decodes `*.vmaxhb` undo-history bytes (an LZFSE-framed binary plist) into a
/// [`VMaxHistoryVmaxhbFile`]. The inverse of
/// [`to_history_vmaxhb_file_bytes`](crate::to_history_vmaxhb_file_bytes).
pub fn from_history_vmaxhb_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhbFile> {
    let decompressed = decompress_lzfse(bytes);
    plist::from_bytes(&decompressed).map_err(Error::Plist)
}
