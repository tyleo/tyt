use crate::{VMaxHistoryStep, VMaxSnapshotId, VMaxValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One edit session in a Voxel Max history file (`sessions[]`): a session id,
/// its ordered [`steps`](Self::steps), and the snapshot identifiers it touched.
/// The scene-snapshot ([`ssnapshots`](Self::ssnapshots)) and object-snapshot
/// ([`osnapshots`](Self::osnapshots)) bodies are an undocumented, per-command
/// shape, so they are held as faithful [`VMaxValue`] trees.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxHistorySession {
    /// Session id.
    pub sid: i64,

    /// Ordered edit steps.
    pub steps: Vec<VMaxHistoryStep>,

    /// Volume-snapshot identifiers touched by this session.
    pub snapshots: Vec<VMaxSnapshotId>,

    /// Scene-snapshot payloads.
    pub ssnapshots: Vec<VMaxValue>,

    /// Object-snapshot payloads.
    pub osnapshots: Vec<VMaxValue>,
}
