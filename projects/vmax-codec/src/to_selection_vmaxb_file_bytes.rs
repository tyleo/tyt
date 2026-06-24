use crate::Result;
use vmax::VMaxSelectionVmaxbFile;

/// Writes a [`VMaxSelectionVmaxbFile`] back to `*.selection.vmaxb` bytes, the
/// inverse of
/// [`from_selection_vmaxb_file_bytes`](crate::from_selection_vmaxb_file_bytes).
/// The preserved selection payload is returned verbatim.
pub fn to_selection_vmaxb_file_bytes(file: &VMaxSelectionVmaxbFile) -> Result<Vec<u8>> {
    Ok(file.0.clone())
}
