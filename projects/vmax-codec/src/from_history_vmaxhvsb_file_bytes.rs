use std::io::Result;
use vmax::VMaxHistoryVmaxhvsbFile;

/// Reads `*.vmaxhvsb` history voxel-snapshot-buffer bytes into a
/// [`VMaxHistoryVmaxhvsbFile`]. The buffer is opaque to this crate, so the bytes
/// are kept verbatim.
pub fn from_history_vmaxhvsb_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhvsbFile> {
    Ok(VMaxHistoryVmaxhvsbFile(bytes.to_vec()))
}
