use crate::{Error, Result, compress_lzfse};
use vmax::VMaxSerdeContentsVmaxbFile;

/// Encodes a [`VMaxSerdeContentsVmaxbFile`] into `contents*.vmaxb` bytes (a
/// binary plist wrapped in an LZFSE block stream), the inverse of
/// [`from_contents_vmaxb_file_bytes`](crate::from_contents_vmaxb_file_bytes).
pub fn to_contents_vmaxb_file_bytes(file: &VMaxSerdeContentsVmaxbFile) -> Result<Vec<u8>> {
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, file).map_err(Error::Plist)?;
    Ok(compress_lzfse(&plist_bytes))
}
