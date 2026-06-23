use std::io::Result;
use vmax::VMaxHistoryVmaxhbFile;

/// Reads `*.vmaxhb` undo-history bytes into a [`VMaxHistoryVmaxhbFile`]. The
/// history stream is opaque to this crate, so the bytes are kept verbatim.
pub fn from_history_vmaxhb_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhbFile> {
    Ok(VMaxHistoryVmaxhbFile(bytes.to_vec()))
}
