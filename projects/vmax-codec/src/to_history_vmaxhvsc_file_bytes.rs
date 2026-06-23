use std::io::Result;
use vmax::VMaxHistoryVmaxhvscFile;

/// Writes a [`VMaxHistoryVmaxhvscFile`] back to `*.vmaxhvsc` bytes — the inverse
/// of [`from_history_vmaxhvsc_file_bytes`](crate::from_history_vmaxhvsc_file_bytes).
/// The preserved sidecar plist is returned verbatim.
pub fn to_history_vmaxhvsc_file_bytes(file: &VMaxHistoryVmaxhvscFile) -> Result<Vec<u8>> {
    Ok(file.0.clone())
}
