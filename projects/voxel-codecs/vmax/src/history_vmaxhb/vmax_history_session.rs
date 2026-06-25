use crate::{VMaxHistoryStep, VMaxSnapshotId, VMaxValue};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// One edit session in a history file (`sessions[]`).
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

    /// Scene-snapshot payloads; undocumented per-command shape, kept as untyped
    /// `VMaxValue` (round-trips unchanged).
    pub ssnapshots: Vec<VMaxValue>,

    /// Object-snapshot payloads; undocumented per-command shape, kept as untyped
    /// `VMaxValue` (round-trips unchanged).
    pub osnapshots: Vec<VMaxValue>,
}
