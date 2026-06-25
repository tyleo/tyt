use crate::VMaxSnapshot;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Decoded `*.vmaxhvsb` history voxel-snapshot buffer: an LZFSE-framed (`bvx2`)
/// companion to a `*.vmaxhb` undo history, sharing its `history{n}` stem (e.g.
/// `history1.vmaxhvsb`), holding the snapshots the history steps reference. The
/// payload is a binary plist array of the same
/// [`VMaxSnapshot`](crate::VMaxSnapshot) shape `contents*.vmaxb` uses.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize), serde(transparent))]
pub struct VMaxHistoryVmaxhvsbFile(pub Vec<VMaxSnapshot>);
