use crate::{Error, Result};
use vmax::VMaxHistoryVmaxhvscFile;

/// Encodes a [`VMaxHistoryVmaxhvscFile`] into `*.vmaxhvsc` bytes (a binary
/// plist, not outer-compressed), the inverse of
/// [`from_history_vmaxhvsc_file_bytes`](crate::from_history_vmaxhvsc_file_bytes).
pub fn to_history_vmaxhvsc_file_bytes(file: &VMaxHistoryVmaxhvscFile) -> Result<Vec<u8>> {
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, file).map_err(Error::Plist)?;
    Ok(plist_bytes)
}
