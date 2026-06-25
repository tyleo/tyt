use crate::VMaxHistorySession;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A decoded `*.vmaxhb` undo-history file: the `sessions` timeline Voxel Max
/// writes per object (named by the object's `hist` reference, e.g.
/// `history1.vmaxhb`) and once for the working scene (`scene.vmaxhb`). The
/// envelope is an LZFSE-framed (`bvx2`) binary plist.
///
/// The session/step skeleton is modeled; the per-command undo/redo payloads
/// inside it are held as [`VMaxValue`](crate::VMaxValue) trees (see
/// [`VMaxHistorySession`](crate::VMaxHistorySession)), so the whole file
/// round-trips without dropping anything.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxHistoryVmaxhbFile {
    /// The edit sessions, in order.
    pub sessions: Vec<VMaxHistorySession>,

    /// Active session id.
    pub asid: i64,
}
