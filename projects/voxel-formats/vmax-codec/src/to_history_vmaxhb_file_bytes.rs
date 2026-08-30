use crate::{Error, Result, compress_lzfse};
use vmax::VMaxHistoryVmaxhbFile;

/// Encodes a [`VMaxHistoryVmaxhbFile`] into `*.vmaxhb` bytes (a binary plist
/// wrapped in an LZFSE block stream), the inverse of
/// [`from_history_vmaxhb_file_bytes`](crate::from_history_vmaxhb_file_bytes).
pub fn to_history_vmaxhb_file_bytes(file: &VMaxHistoryVmaxhbFile) -> Result<Vec<u8>> {
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, file).map_err(Error::Plist)?;
    Ok(compress_lzfse(&plist_bytes))
}
