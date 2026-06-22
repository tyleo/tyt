use crate::compress_lzfse;
use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxContentsVmaxbFile;

/// Encodes a [`VMaxContentsVmaxbFile`] into `contents*.vmaxb` bytes (a binary plist
/// wrapped in an LZFSE block stream) — the inverse of
/// [`from_vmaxb_bytes`](crate::from_vmaxb_bytes).
pub fn to_vmaxb_bytes(file: &VMaxContentsVmaxbFile) -> Result<Vec<u8>> {
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, file)
        .map_err(|e| IOError::new(ErrorKind::InvalidData, e))?;
    Ok(compress_lzfse(&plist_bytes))
}
