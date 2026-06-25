use crate::Result;
use vmax::VMaxHistoryVmaxhvsbFile;

/// Writes a [`VMaxHistoryVmaxhvsbFile`] back to `*.vmaxhvsb` bytes, the
/// inverse of
/// [`from_history_vmaxhvsb_file_bytes`](crate::from_history_vmaxhvsb_file_bytes).
/// The preserved snapshot buffer is returned verbatim.
pub fn to_history_vmaxhvsb_file_bytes(file: &VMaxHistoryVmaxhvsbFile) -> Result<Vec<u8>> {
    Ok(file.0.clone())
}
