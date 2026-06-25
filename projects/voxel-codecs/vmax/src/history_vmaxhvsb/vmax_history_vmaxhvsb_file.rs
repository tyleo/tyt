use crate::VMaxSnapshot;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A decoded `*.vmaxhvsb` history voxel-snapshot buffer: an LZFSE-framed
/// (`bvx2`) companion to a `*.vmaxhb` undo history, sharing its `history{n}`
/// stem (e.g. `history1.vmaxhvsb`), holding the voxel snapshots the history
/// steps reference. The payload is a binary plist array of the same
/// [`VMaxSnapshot`](crate::VMaxSnapshot) shape a `contents*.vmaxb`
/// uses, so it decodes into typed snapshots and round-trips every field.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(transparent))]
pub struct VMaxHistoryVmaxhvsbFile(pub Vec<VMaxSnapshot>);
