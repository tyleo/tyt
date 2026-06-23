use std::io::Result;
use vmax::VMaxSelectionVmaxbFile;

/// Reads `*.selection.vmaxb` saved-selection bytes into a
/// [`VMaxSelectionVmaxbFile`]. The selection payload is opaque to this crate,
/// so the bytes are kept verbatim.
pub fn from_selection_vmaxb_file_bytes(bytes: &[u8]) -> Result<VMaxSelectionVmaxbFile> {
    Ok(VMaxSelectionVmaxbFile(bytes.to_vec()))
}
