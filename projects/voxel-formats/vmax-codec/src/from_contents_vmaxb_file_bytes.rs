use crate::{DecodeVMaxPlist, DecompressLzfse, Error, Result, decompress_lzfse_or_raw};
use vmax::VMaxContentsVmaxbFile;

/// Decodes `contents*.vmaxb` bytes (an LZFSE-framed binary plist) into a
/// [`VMaxContentsVmaxbFile`] through `dependencies`.
pub fn from_contents_vmaxb_file_bytes<D: DecompressLzfse + DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxContentsVmaxbFile> {
    let plist_bytes = decompress_lzfse_or_raw(dependencies, bytes);
    dependencies
        .decode_contents_vmaxb(&plist_bytes)
        .map_err(Error::Plist)
}
