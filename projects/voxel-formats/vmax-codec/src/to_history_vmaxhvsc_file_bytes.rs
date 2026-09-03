use crate::{EncodeVMaxPlist, Error, Result};
use vmax::VMaxHistoryVmaxhvscFile;

/// Encodes a [`VMaxHistoryVmaxhvscFile`] into `*.vmaxhvsc` bytes (a binary
/// plist, not outer-compressed) through `dependencies`, the inverse of
/// [`from_history_vmaxhvsc_file_bytes`](crate::from_history_vmaxhvsc_file_bytes).
pub fn to_history_vmaxhvsc_file_bytes<D: EncodeVMaxPlist>(
    dependencies: &D,
    file: &VMaxHistoryVmaxhvscFile,
) -> Result<Vec<u8>> {
    dependencies
        .encode_history_vmaxhvsc(file)
        .map_err(Error::Plist)
}
