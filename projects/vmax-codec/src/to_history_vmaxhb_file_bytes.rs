use crate::Result;
use vmax::VMaxHistoryVmaxhbFile;

/// Writes a [`VMaxHistoryVmaxhbFile`] back to `*.vmaxhb` bytes, the inverse of
/// [`from_history_vmaxhb_file_bytes`](crate::from_history_vmaxhb_file_bytes).
/// The preserved history stream is returned verbatim.
pub fn to_history_vmaxhb_file_bytes(file: &VMaxHistoryVmaxhbFile) -> Result<Vec<u8>> {
    Ok(file.0.clone())
}
