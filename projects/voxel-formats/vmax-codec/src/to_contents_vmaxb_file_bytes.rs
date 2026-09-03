use crate::{CompressLzfse, EncodeVMaxPlist, Error, Result};
use vmax::VMaxContentsVmaxbFile;

/// Encodes a [`VMaxContentsVmaxbFile`] into `contents*.vmaxb` bytes (a
/// binary plist wrapped in an LZFSE block stream) through `dependencies`, the
/// inverse of
/// [`from_contents_vmaxb_file_bytes`](crate::from_contents_vmaxb_file_bytes).
pub fn to_contents_vmaxb_file_bytes<D: CompressLzfse + EncodeVMaxPlist>(
    dependencies: &D,
    file: &VMaxContentsVmaxbFile,
) -> Result<Vec<u8>> {
    let plist_bytes = dependencies
        .encode_contents_vmaxb(file)
        .map_err(Error::Plist)?;
    Ok(dependencies.compress_lzfse(&plist_bytes))
}
