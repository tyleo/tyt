use crate::{Error, Result};
use vmax::VMaxHistoryVmaxhvscFile;

/// Decodes `*.vmaxhvsc` history voxel-snapshot-sidecar bytes (a binary plist
/// array, not outer-compressed) into a [`VMaxHistoryVmaxhvscFile`]. The inverse
/// of
/// [`to_history_vmaxhvsc_file_bytes`](crate::to_history_vmaxhvsc_file_bytes).
pub fn from_history_vmaxhvsc_file_bytes(bytes: &[u8]) -> Result<VMaxHistoryVmaxhvscFile> {
    plist::from_bytes(bytes).map_err(Error::Plist)
}
