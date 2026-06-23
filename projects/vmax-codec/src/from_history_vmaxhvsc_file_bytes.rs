use std::io::Result;
use vmax::VMaxHistoryVmaxhvscFile;

/// Reads `*.vmaxhvsc` history voxel-snapshot-sidecar bytes into a
/// [`VMaxHistoryVmaxhvscFile`]. The sidecar plist is opaque to this crate, so the
/// bytes are kept verbatim.
pub fn from_history_vmaxhvsc_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhvscFile> {
    Ok(VMaxHistoryVmaxhvscFile(bytes.to_vec()))
}
