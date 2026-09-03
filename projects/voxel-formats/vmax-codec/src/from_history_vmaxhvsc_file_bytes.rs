use crate::{DecodeVMaxPlist, Error, Result};
use vmax::VMaxHistoryVmaxhvscFile;

/// Decodes `*.vmaxhvsc` history voxel-snapshot-sidecar bytes (a binary plist
/// array, not outer-compressed) into a [`VMaxHistoryVmaxhvscFile`] through
/// `dependencies`. The inverse of
/// [`to_history_vmaxhvsc_file_bytes`](crate::to_history_vmaxhvsc_file_bytes).
pub fn from_history_vmaxhvsc_file_bytes<D: DecodeVMaxPlist>(
    dependencies: &D,
    bytes: &[u8],
) -> Result<VMaxHistoryVmaxhvscFile> {
    dependencies
        .decode_history_vmaxhvsc(bytes)
        .map_err(Error::Plist)
}
