use crate::VMaxValue;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A decoded `*.vmaxhvsc` history voxel-snapshot sidecar: a binary plist
/// (`bplist00`, not outer-compressed) companion to a `*.vmaxhvsb` snapshot
/// buffer, sharing its `history{n}` stem (e.g. `history1.vmaxhvsc`), recording
/// the compressed-snapshot batches Voxel Max keeps alongside the buffer.
///
/// The payload is a plist array of batch records whose shape is undocumented
/// (and empty in every sample observed), so each entry is held as a faithful
/// [`VMaxValue`](crate::VMaxValue) tree and round-trips without being
/// interpreted.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(transparent))]
pub struct VMaxHistoryVmaxhvscFile(pub Vec<VMaxValue>);
