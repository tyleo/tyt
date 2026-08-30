use crate::{Error, Result, compress_lzfse};
use vmax::VMaxHistoryVmaxhvsbFile;

/// Encodes a [`VMaxHistoryVmaxhvsbFile`] into `*.vmaxhvsb` bytes (a binary
/// plist wrapped in an LZFSE block stream), the inverse of
/// [`from_history_vmaxhvsb_file_bytes`](crate::from_history_vmaxhvsb_file_bytes).
pub fn to_history_vmaxhvsb_file_bytes(file: &VMaxHistoryVmaxhvsbFile) -> Result<Vec<u8>> {
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, file).map_err(Error::Plist)?;
    Ok(compress_lzfse(&plist_bytes))
}
