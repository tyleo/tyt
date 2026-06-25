use crate::{Error, Result, decompress_lzfse};
use vmax::VMaxContentsVmaxbFile;

/// Decodes `contents*.vmaxb` bytes (an LZFSE-framed binary plist) into a
/// [`VMaxContentsVmaxbFile`].
pub fn from_contents_vmaxb_file_bytes(bytes: &[u8]) -> Result<VMaxContentsVmaxbFile> {
    let decompressed = decompress_lzfse(bytes);
    plist::from_bytes(&decompressed).map_err(Error::Plist)
}
