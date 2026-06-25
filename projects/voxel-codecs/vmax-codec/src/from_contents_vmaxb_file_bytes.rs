use crate::{Error, Result, decompress_lzfse};
use vmax::VMaxSerdeContentsVmaxbFile;

/// Decodes `contents*.vmaxb` bytes (an LZFSE-framed binary plist) into a
/// [`VMaxSerdeContentsVmaxbFile`].
pub fn from_contents_vmaxb_file_bytes(bytes: &[u8]) -> Result<VMaxSerdeContentsVmaxbFile> {
    let decompressed = decompress_lzfse(bytes);
    plist::from_bytes(&decompressed).map_err(Error::Plist)
}
