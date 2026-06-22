use crate::decompress_lzfse;
use std::io::{Error as IOError, ErrorKind, Result};
use vmax::VMaxContentsVmaxbFile;

/// Decodes `contents*.vmaxb` bytes (an LZFSE-framed binary plist) into a
/// [`VMaxContentsVmaxbFile`].
pub fn from_vmaxb_bytes(bytes: &[u8]) -> Result<VMaxContentsVmaxbFile> {
    let decompressed = decompress_lzfse(bytes);
    plist::from_bytes(&decompressed).map_err(|e| IOError::new(ErrorKind::InvalidData, e))
}
